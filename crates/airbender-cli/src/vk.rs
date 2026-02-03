use anyhow::{Context, Result};
use execution_utils::unified_circuit::verify_proof_in_unified_layer;
use execution_utils::unrolled::{UnrolledProgramProof, UnrolledProgramSetup};
use execution_utils::setups;
use risc_v_simulator::cycle::IWithoutByteAccessIsaConfigWithDelegation;
use sha3::Digest;
use std::fs;
use std::path::Path;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UnifiedVkFile {
    pub app_bin_hash: [u8; 32],
    pub unified_setup: UnrolledProgramSetup,
    pub unified_layouts: setups::CompiledCircuitsSet,
}

pub fn generate_vk(app_bin: &Path, output: &Path) -> Result<()> {
    let app_bin_bytes =
        fs::read(app_bin).with_context(|| format!("failed to read {}", app_bin.display()))?;
    let app_bin_hash: [u8; 32] = sha3::Keccak256::digest(&app_bin_bytes).into();

    let (binary, binary_u32) =
        setups::pad_binary(execution_utils::unrolled_gpu::RECURSION_UNIFIED_BIN.to_vec());
    let (text, _) = setups::pad_binary(execution_utils::unrolled_gpu::RECURSION_UNIFIED_TXT.to_vec());

    tracing::info!("Computing unified recursion VKs");
    let unified_setup = execution_utils::unified_circuit::compute_unified_setup_for_machine_configuration::<
        IWithoutByteAccessIsaConfigWithDelegation,
    >(&binary, &text);
    let unified_layouts = execution_utils::setups::get_unified_circuit_artifact_for_machine_type::<
        IWithoutByteAccessIsaConfigWithDelegation,
    >(&binary_u32);

    let vk_file = UnifiedVkFile {
        app_bin_hash,
        unified_setup,
        unified_layouts,
    };
    let encoded = bincode::serde::encode_to_vec(&vk_file, bincode::config::standard())?;
    fs::write(output, encoded)
        .with_context(|| format!("failed to write VK file to {}", output.display()))?;
    tracing::info!("VKs written to {}", output.display());
    Ok(())
}

pub fn verify_proof(proof_path: &Path, vk_path: &Path) -> Result<()> {
    let proof = read_bincode::<UnrolledProgramProof>(proof_path)
        .context("failed to decode proof")?;
    let vk_file = read_bincode::<UnifiedVkFile>(vk_path).context("failed to decode VK file")?;

    tracing::info!("Verifying proof");
    let result = verify_proof_in_unified_layer(
        &proof,
        &vk_file.unified_setup,
        &vk_file.unified_layouts,
        false,
    )
    .map_err(|_| anyhow::anyhow!("proof verification failed"))?;
    tracing::info!("Proof verified successfully, output={result:?}");
    Ok(())
}

fn read_bincode<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    let (decoded, read_len) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard())?;
    if read_len != bytes.len() {
        tracing::warn!(
            "bincode decoded {} bytes but file is {} bytes",
            read_len,
            bytes.len()
        );
    }
    Ok(decoded)
}
