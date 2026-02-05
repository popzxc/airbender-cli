use anyhow::{Context, Result};
use execution_utils::unified_circuit::verify_proof_in_unified_layer;
use execution_utils::unrolled::{
    compute_setup_for_machine_configuration, get_unrolled_circuits_artifacts_for_machine_type,
    verify_unrolled_layer_proof, UnrolledProgramProof, UnrolledProgramSetup,
};
use execution_utils::unrolled_gpu::UnrolledProverLevel;
use execution_utils::setups;
use risc_v_simulator::cycle::{
    IMStandardIsaConfigWithUnsignedMulDiv, IWithoutByteAccessIsaConfigWithDelegation,
};
use sha3::Digest;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UnifiedVkFile {
    pub app_bin_hash: [u8; 32],
    pub unified_setup: UnrolledProgramSetup,
    pub unified_layouts: setups::CompiledCircuitsSet,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UnrolledVkFile {
    pub app_bin_hash: [u8; 32],
    pub setup: UnrolledProgramSetup,
    pub compiled_layouts: setups::CompiledCircuitsSet,
}

pub fn generate_vk(app_bin: &Path, output: &Path, level: UnrolledProverLevel) -> Result<()> {
    match level {
        UnrolledProverLevel::RecursionUnified => generate_unified_vk(app_bin, output),
        UnrolledProverLevel::Base | UnrolledProverLevel::RecursionUnrolled => {
            generate_unrolled_vk(app_bin, output, level)
        }
    }
}

fn generate_unified_vk(app_bin: &Path, output: &Path) -> Result<()> {
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

fn generate_unrolled_vk(
    app_bin: &Path,
    output: &Path,
    level: UnrolledProverLevel,
) -> Result<()> {
    let base_path = strip_bin_suffix(app_bin)?;
    let app_bin_path = PathBuf::from(format!("{base_path}.bin"));
    let app_text_path = PathBuf::from(format!("{base_path}.text"));

    let app_bin_bytes =
        fs::read(&app_bin_path).with_context(|| format!("failed to read {}", app_bin_path.display()))?;
    let app_bin_hash: [u8; 32] = sha3::Keccak256::digest(&app_bin_bytes).into();

    let (binary, binary_u32, text) = match level {
        UnrolledProverLevel::Base => {
            let (binary, binary_u32) = setups::read_and_pad_binary(&app_bin_path);
            let (text, _) = setups::read_and_pad_binary(&app_text_path);
            (binary, binary_u32, text)
        }
        UnrolledProverLevel::RecursionUnrolled => {
            let (binary, binary_u32) =
                setups::pad_binary(execution_utils::unrolled_gpu::RECURSION_UNROLLED_BIN.to_vec());
            let (text, _) =
                setups::pad_binary(execution_utils::unrolled_gpu::RECURSION_UNROLLED_TXT.to_vec());
            (binary, binary_u32, text)
        }
        UnrolledProverLevel::RecursionUnified => {
            return Err(anyhow::anyhow!("unified VKs are generated separately"));
        }
    };

    let (setup, compiled_layouts) = match level {
        UnrolledProverLevel::Base => {
            let setup = compute_setup_for_machine_configuration::<
                IMStandardIsaConfigWithUnsignedMulDiv,
            >(&binary, &text);
            let compiled_layouts = get_unrolled_circuits_artifacts_for_machine_type::<
                IMStandardIsaConfigWithUnsignedMulDiv,
            >(&binary_u32);
            (setup, compiled_layouts)
        }
        UnrolledProverLevel::RecursionUnrolled => {
            let setup = compute_setup_for_machine_configuration::<
                IWithoutByteAccessIsaConfigWithDelegation,
            >(&binary, &text);
            let compiled_layouts = get_unrolled_circuits_artifacts_for_machine_type::<
                IWithoutByteAccessIsaConfigWithDelegation,
            >(&binary_u32);
            (setup, compiled_layouts)
        }
        UnrolledProverLevel::RecursionUnified => {
            return Err(anyhow::anyhow!("unified VKs are generated separately"));
        }
    };

    let vk_file = UnrolledVkFile {
        app_bin_hash,
        setup,
        compiled_layouts,
    };
    let encoded = bincode::serde::encode_to_vec(&vk_file, bincode::config::standard())?;
    fs::write(output, encoded)
        .with_context(|| format!("failed to write VK file to {}", output.display()))?;
    tracing::info!("VKs written to {}", output.display());
    Ok(())
}

pub fn verify_proof(
    proof_path: &Path,
    vk_path: &Path,
    level: UnrolledProverLevel,
) -> Result<()> {
    let proof = read_bincode::<UnrolledProgramProof>(proof_path)
        .context("failed to decode proof")?;
    tracing::info!("Verifying proof");
    match level {
        UnrolledProverLevel::RecursionUnified => {
            let vk_file =
                read_bincode::<UnifiedVkFile>(vk_path).context("failed to decode VK file")?;
            let result = verify_proof_in_unified_layer(
                &proof,
                &vk_file.unified_setup,
                &vk_file.unified_layouts,
                false,
            )
            .map_err(|_| anyhow::anyhow!("proof verification failed"))?;
            tracing::info!("Proof verified successfully, output={result:?}");
        }
        UnrolledProverLevel::Base | UnrolledProverLevel::RecursionUnrolled => {
            let vk_file =
                read_bincode::<UnrolledVkFile>(vk_path).context("failed to decode VK file")?;
            let is_base_layer = level == UnrolledProverLevel::Base;
            let result = verify_unrolled_layer_proof(
                &proof,
                &vk_file.setup,
                &vk_file.compiled_layouts,
                is_base_layer,
            )
            .map_err(|_| anyhow::anyhow!("proof verification failed"))?;
            tracing::info!("Proof verified successfully, output={result:?}");
        }
    }
    Ok(())
}

fn strip_bin_suffix(path: &Path) -> Result<String> {
    let path_str = path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("app path is not valid UTF-8"))?;
    if let Some(stripped) = path_str.strip_suffix(".bin") {
        Ok(stripped.to_string())
    } else {
        Ok(path_str.to_string())
    }
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
