use move_core_types::account_address::AccountAddress;
use move_vm_runtime::native_functions::{make_table_from_iter, NativeFunction};
use move_vm_types::natives::function::NativeResult;
use move_vm_types::{pop_arg, values::{Value, VectorRef}};
use move_vm_types::natives::function::PartialVMResult;
use smallvec::smallvec;
use std::{collections::VecDeque, sync::Arc};

use k256::ecdsa::{signature::Verifier as K256Verifier, Signature as K256Signature, VerifyingKey as K256VerifyingKey};
use k256::PublicKey as K256PublicKey;
use secp256k1::{Message as SecpMessage, Secp256k1, ecdsa::RecoverableSignature as SecpRecoverableSignature, ecdsa::RecoveryId as SecpRecoveryId, PublicKey as SecpPublicKey};
use p256::ecdsa::{Signature as P256Signature, VerifyingKey as P256VerifyingKey};
use sha3::{Keccak256, Digest};
use sha2::Sha256;
use k256::elliptic_curve::sec1::ToEncodedPoint;

use ed25519_dalek::{VerifyingKey as EdPublicKey, Signature as EdSignature};
use std::convert::TryInto;

// Build a NativeFunction easily
fn make_native<F>(f: F) -> NativeFunction
where
    F: Fn(&mut move_vm_runtime::native_functions::NativeContext, Vec<move_vm_types::loaded_data::runtime_types::Type>, VecDeque<move_vm_types::values::Value>) -> PartialVMResult<NativeResult>
        + Send
        + Sync
        + 'static,
{
    Arc::new(f)
}

pub fn all_natives(move_addr: AccountAddress) -> move_vm_runtime::native_functions::NativeFunctionTable {
    let mut natives = vec![];

    // ecdsa_k1::ecrecover(signature: vector<u8>, msg: vector<u8>, hash: u8): vector<u8>
    let ecrecover_native = make_native(move |context, _ty_args, mut arguments| -> PartialVMResult<NativeResult> {
        use move_vm_types::natives::function::NativeResult as NR;
        
        // pop in reverse order: hash, msg, signature
        let hash_type: u8 = pop_arg!(arguments, u8);
        let msg_ref: VectorRef = pop_arg!(arguments, VectorRef);
        let signature_ref: VectorRef = pop_arg!(arguments, VectorRef);
        let msg: Vec<u8> = msg_ref.as_bytes_ref().to_vec();
        let signature: Vec<u8> = signature_ref.as_bytes_ref().to_vec();

        // simple gas cost = 0
        // Validate signature length
        if signature.len() != 65 {
            return Ok(NR::err(context.gas_used(), 2)); // ErrorInvalidSignature
        }

        // hash
        let msg_hash = if hash_type == 0u8 {
            // keccak256
            use sha3::Digest;
            Keccak256::digest(&msg).to_vec()
        } else {
            use sha2::Digest;
            Sha256::digest(&msg).to_vec()
        };

        // Recover: use secp256k1 to recover public key from (r,s,v)
        if signature.len() != 65 {
            return Ok(NR::err(context.gas_used(), 1));
        }
        let mut sig64 = [0u8; 64];
        sig64.copy_from_slice(&signature[0..64]);
        let v = signature[64];
        // RecoveryId implements TryFrom<i32> in this secp256k1 version
        let rec_id = match SecpRecoveryId::try_from((v % 4) as i32) {
            Ok(r) => r,
            Err(_) => return Ok(NR::err(context.gas_used(), 1)),
        };
        let secp_sig = match SecpRecoverableSignature::from_compact(&sig64, rec_id) {
            Ok(s) => s,
            Err(_) => return Ok(NR::err(context.gas_used(), 1)),
        };
        let secp = Secp256k1::new();
        // Message expects 32-byte hash
        let msg32: [u8;32] = if msg_hash.len() == 32 { msg_hash.clone().try_into().unwrap() } else { let mut a=[0u8;32]; a.copy_from_slice(&msg_hash[0..32]); a };
        let message = match SecpMessage::from_slice(&msg32) { Ok(m) => m, Err(_) => return Ok(NR::err(context.gas_used(), 1)) };
        let pubkey = match secp.recover_ecdsa(&message, &secp_sig) {
            Ok(pk) => pk,
            Err(_) => return Ok(NR::err(context.gas_used(), 1)),
        };
        // Convert secp public key to uncompressed bytes and return
        let out = SecpPublicKey::from(pubkey).serialize_uncompressed().to_vec();
        Ok(NR::ok(context.gas_used(), smallvec![Value::vector_u8(out)]))
    });

    // ecdsa_k1::decompress_pubkey(pubkey: vector<u8>): vector<u8>
    let decompress_native = make_native(move |context, _ty_args, mut arguments| -> PartialVMResult<NativeResult> {
        use move_vm_types::natives::function::NativeResult as NR;
        let pubkey_ref: VectorRef = pop_arg!(arguments, VectorRef);
        let pubkey: Vec<u8> = pubkey_ref.as_bytes_ref().to_vec();

        // Accept compressed (33) or uncompressed (65) and return uncompressed 65
        let pk_res = K256PublicKey::from_sec1_bytes(&pubkey);
        if pk_res.is_err() {
            return Ok(NR::err(context.gas_used(), 3)); // ErrorInvalidPubKey
        }
        let pk = pk_res.unwrap();
        let ep = pk.to_encoded_point(false);
        let out = ep.as_bytes().to_vec();
        Ok(NR::ok(context.gas_used(), smallvec![Value::vector_u8(out)]))
    });

    // ecdsa_k1::verify(signature, public_key, msg, hash) -> bool
    let verify_k1 = make_native(move |context, _ty_args, mut arguments| -> PartialVMResult<NativeResult> {
        use move_vm_types::natives::function::NativeResult as NR;
        let hash_type: u8 = pop_arg!(arguments, u8);
        let msg_ref: VectorRef = pop_arg!(arguments, VectorRef);
        let public_key_ref: VectorRef = pop_arg!(arguments, VectorRef);
        let signature_ref: VectorRef = pop_arg!(arguments, VectorRef);
        let msg: Vec<u8> = msg_ref.as_bytes_ref().to_vec();
        let public_key: Vec<u8> = public_key_ref.as_bytes_ref().to_vec();
        let signature: Vec<u8> = signature_ref.as_bytes_ref().to_vec();

        if signature.is_empty() {
            return Ok(NR::err(context.gas_used(), 2)); // ErrorInvalidSignature
        }

        // Hash
        let msg_hash = if hash_type == 0u8 {
            Keccak256::digest(&msg).to_vec()
        } else {
            Sha256::digest(&msg).to_vec()
        };

        // parse pubkey (allow compressed/uncompressed)
        let vk = match K256VerifyingKey::from_sec1_bytes(&public_key) {
            Ok(v) => v,
            Err(_) => return Ok(NR::err(context.gas_used(), 3)),
        };

        // parse signature: try DER then raw 64
        let sig = if let Ok(s) = K256Signature::from_der(&signature) {
            s
        } else if signature.len() == 64 {
            // try raw 64-bytes signature
            match K256Signature::try_from(&signature[..]) {
                Ok(s) => s,
                Err(_) => return Ok(NR::ok(context.gas_used(), smallvec![Value::bool(false)])),
            }
        } else {
            return Ok(NR::ok(context.gas_used(), smallvec![Value::bool(false)]));
        };

        // Use digest-aware verification: verify the hashed message
        let verified = if hash_type == 0u8 {
            // Keccak256
            use k256::ecdsa::signature::DigestVerifier;
            let mut hasher = Keccak256::new();
            hasher.update(&msg);
            vk.verify_digest(hasher, &sig).is_ok()
        } else {
            use k256::ecdsa::signature::DigestVerifier;
            let mut hasher = Sha256::new();
            hasher.update(&msg);
            vk.verify_digest(hasher, &sig).is_ok()
        };

        move_vm_types::natives::function::NativeResult::map_partial_vm_result_one(context.gas_used(), Ok(Value::bool(verified)))
    });

    // ecdsa_r1 (P-256) verify(signature, public_key, msg, hash) -> bool
    let verify_r1 = make_native(move |context, _ty_args, mut arguments| -> PartialVMResult<NativeResult> {
        use move_vm_types::natives::function::NativeResult as NR;
        let hash_type: u8 = pop_arg!(arguments, u8);
        let msg_ref: VectorRef = pop_arg!(arguments, VectorRef);
        let public_key_ref: VectorRef = pop_arg!(arguments, VectorRef);
        let signature_ref: VectorRef = pop_arg!(arguments, VectorRef);
        let msg: Vec<u8> = msg_ref.as_bytes_ref().to_vec();
        let public_key: Vec<u8> = public_key_ref.as_bytes_ref().to_vec();
        let signature: Vec<u8> = signature_ref.as_bytes_ref().to_vec();

        if signature.is_empty() {
            return Ok(NR::err(context.gas_used(), 2)); // ErrorInvalidSignature
        }

        // Only SHA256 is supported for P-256 in Move wrapper, but accept hash_type selection defensively
        let vk = match P256VerifyingKey::from_sec1_bytes(&public_key) {
            Ok(v) => v,
            Err(_) => return Ok(NR::err(context.gas_used(), 3)),
        };

        let sig = if let Ok(s) = P256Signature::from_der(&signature) {
            s
        } else if signature.len() == 64 {
            match P256Signature::try_from(&signature[..]) {
                Ok(s) => s,
                Err(_) => return Ok(NR::ok(context.gas_used(), smallvec![Value::bool(false)])),
            }
        } else {
            return Ok(NR::ok(context.gas_used(), smallvec![Value::bool(false)]));
        };

        // Hash then verify via digest-aware API
        let verified = if hash_type == 0u8 {
            // Keccak (not typical for P-256) – still allow
            let mut hasher = Keccak256::new();
            hasher.update(&msg);
            use p256::ecdsa::signature::DigestVerifier;
            vk.verify_digest(hasher, &sig).is_ok()
        } else {
            let mut hasher = Sha256::new();
            hasher.update(&msg);
            use p256::ecdsa::signature::DigestVerifier;
            vk.verify_digest(hasher, &sig).is_ok()
        };

        Ok(NR::ok(context.gas_used(), smallvec![Value::bool(verified)]))
    });

    // ed25519::verify(signature, public_key, msg) -> bool
    let ed25519_verify = make_native(move |context, _ty_args, mut arguments| -> PartialVMResult<NativeResult> {
        use move_vm_types::natives::function::NativeResult as NR;
        // Pop arguments (may return PartialVMError via the macro)
        let msg_ref: VectorRef = pop_arg!(arguments, VectorRef);
        let public_key_ref: VectorRef = pop_arg!(arguments, VectorRef);
        let signature_ref: VectorRef = pop_arg!(arguments, VectorRef);
        let msg: Vec<u8> = msg_ref.as_bytes_ref().to_vec();
        let public_key: Vec<u8> = public_key_ref.as_bytes_ref().to_vec();
        let signature: Vec<u8> = signature_ref.as_bytes_ref().to_vec();

        // Wrap verification in a panic catcher to avoid propagating panics into the VM
        let result = std::panic::catch_unwind(|| {
            if public_key.len() != 32 || signature.len() != 64 {
                return false;
            }

            let pk_arr: [u8; 32] = match public_key.as_slice().try_into() {
                Ok(a) => a,
                Err(_) => return false,
            };
            let pk = match EdPublicKey::from_bytes(&pk_arr) {
                Ok(p) => p,
                Err(_) => return false,
            };

            let sig_arr: [u8; 64] = match signature.as_slice().try_into() {
                Ok(a) => a,
                Err(_) => return false,
            };
            let sig = EdSignature::from_bytes(&sig_arr);

            pk.verify(&msg, &sig).is_ok()
        });

        let verified = match result {
            Ok(b) => b,
            Err(_) => false,
        };

        Ok(NR::ok(context.gas_used(), smallvec![Value::bool(verified)]))
    });

    // Register functions under module names
    natives.push(("ecdsa_k1".to_string(), "ecrecover".to_string(), ecrecover_native));
    natives.push(("ecdsa_k1".to_string(), "decompress_pubkey".to_string(), decompress_native));
    natives.push(("ecdsa_k1".to_string(), "verify".to_string(), verify_k1));
    natives.push(("ecdsa_r1".to_string(), "native_verify".to_string(), verify_r1));
    natives.push(("ed25519".to_string(), "verify".to_string(), ed25519_verify));

    make_table_from_iter(move_addr, natives.into_iter().map(|(m,f,func)| (m.into_boxed_str(), f.into_boxed_str(), func)))
}
