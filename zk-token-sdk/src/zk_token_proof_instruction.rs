//! Instructions provided by the ZkToken Proof program
pub use crate::instruction::*;
use {
    bytemuck::bytes_of,
    num_derive::{FromPrimitive, ToPrimitive},
    num_traits::{FromPrimitive, ToPrimitive},
    solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    },
};

#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum ProofInstruction {
    /// Close a zero-knowledge proof context state.
    ///
    /// Accounts expected by this instruction:
    ///   0. `[writable]` The proof context account to close
    ///   1. `[writable]` The destination account for lamports
    ///   2. `[signer]` The context account's owner
    ///
    /// Data expected by this instruction:
    ///   None
    ///
    CloseContextState,

    /// Verify a zero-balance proof.
    ///
    /// This instruction can be configured to optionally create a proof context state account.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   * Creating a proof context account
    ///   0. `[writable]` The proof context account
    ///   1. `[]` The proof context account owner
    ///
    ///   * Otherwise
    ///   None
    ///
    /// Data expected by this instruction:
    ///   `ZeroBalanceProofData`
    ///
    VerifyZeroBalance,

    /// Verify a withdraw zero-knowledge proof.
    ///
    /// This instruction can be configured to optionally create a proof context state account.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   * Creating a proof context account
    ///   0. `[writable]` The proof context account
    ///   1. `[]` The proof context account owner
    ///
    ///   * Otherwise
    ///   None
    ///
    /// Data expected by this instruction:
    ///   `WithdrawData`
    ///
    VerifyWithdraw,

    /// Verify a ciphertext-ciphertext equality proof.
    ///
    /// This instruction can be configured to optionally create a proof context state account.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   * Creating a proof context account
    ///   0. `[writable]` The proof context account
    ///   1. `[]` The proof context account owner
    ///
    ///   * Otherwise
    ///   None
    ///
    /// Data expected by this instruction:
    ///   `CiphertextCiphertextEqualityProofData`
    ///
    VerifyCiphertextCiphertextEquality,

    /// Verify a transfer zero-knowledge proof.
    ///
    /// This instruction can be configured to optionally create a proof context state account.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   * Creating a proof context account
    ///   0. `[writable]` The proof context account
    ///   1. `[]` The proof context account owner
    ///
    ///   * Otherwise
    ///   None
    ///
    /// Data expected by this instruction:
    ///   `TransferData`
    ///
    VerifyTransfer,

    /// Verify a transfer with fee zero-knowledge proof.
    ///
    /// This instruction can be configured to optionally create a proof context state account.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   * Creating a proof context account
    ///   0. `[writable]` The proof context account
    ///   1. `[]` The proof context account owner
    ///
    ///   * Otherwise
    ///   None
    ///
    /// Data expected by this instruction:
    ///   `TransferWithFeeData`
    ///
    VerifyTransferWithFee,

    /// Verify a pubkey validity zero-knowledge proof.
    ///
    /// This instruction can be configured to optionally create a proof context state account.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   * Creating a proof context account
    ///   0. `[writable]` The proof context account
    ///   1. `[]` The proof context account owner
    ///
    ///   * Otherwise
    ///   None
    ///
    /// Data expected by this instruction:
    ///   `PubkeyValidityData`
    ///
    VerifyPubkeyValidity,
}

/// Pubkeys associated with a context state account to be used as parameters to functions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContextStateInfo<'a> {
    pub context_state_account: &'a Pubkey,
    pub context_state_authority: &'a Pubkey,
}

/// Create a `CloseContextState` instruction.
pub fn close_context_state(
    context_state_info: ContextStateInfo,
    destination_account: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*context_state_info.context_state_account, false),
        AccountMeta::new(*destination_account, false),
        AccountMeta::new_readonly(*context_state_info.context_state_authority, true),
    ];

    let data = vec![ToPrimitive::to_u8(&ProofInstruction::CloseContextState).unwrap()];

    Instruction {
        program_id: crate::zk_token_proof_program::id(),
        accounts,
        data,
    }
}

/// Create a `VerifyZeroBalance` instruction.
pub fn verify_zero_balance(
    context_state_info: Option<ContextStateInfo>,
    proof_data: &ZeroBalanceProofData,
) -> Instruction {
    ProofInstruction::VerifyZeroBalance.encode_verify_proof(context_state_info, proof_data)
}

/// Create a `VerifyWithdraw` instruction.
pub fn verify_withdraw(
    context_state_info: Option<ContextStateInfo>,
    proof_data: &WithdrawData,
) -> Instruction {
    ProofInstruction::VerifyWithdraw.encode_verify_proof(context_state_info, proof_data)
}

/// Create a `VerifyCiphertextCiphertextEquality` instruction.
pub fn verify_ciphertext_ciphertext_equality(
    context_state_info: Option<ContextStateInfo>,
    proof_data: &CiphertextCiphertextEqualityProofData,
) -> Instruction {
    ProofInstruction::VerifyCiphertextCiphertextEquality
        .encode_verify_proof(context_state_info, proof_data)
}

/// Create a `VerifyTransfer` instruction.
pub fn verify_transfer(
    context_state_info: Option<ContextStateInfo>,
    proof_data: &TransferData,
) -> Instruction {
    ProofInstruction::VerifyTransfer.encode_verify_proof(context_state_info, proof_data)
}

/// Create a `VerifyTransferWithFee` instruction.
pub fn verify_transfer_with_fee(
    context_state_info: Option<ContextStateInfo>,
    proof_data: &TransferWithFeeData,
) -> Instruction {
    ProofInstruction::VerifyTransferWithFee.encode_verify_proof(context_state_info, proof_data)
}

/// Create a `VerifyPubkeyValidity` instruction.
pub fn verify_pubkey_validity(
    context_state_info: Option<ContextStateInfo>,
    proof_data: &PubkeyValidityData,
) -> Instruction {
    ProofInstruction::VerifyPubkeyValidity.encode_verify_proof(context_state_info, proof_data)
}

impl ProofInstruction {
    pub fn encode_verify_proof<T, U>(
        &self,
        context_state_info: Option<ContextStateInfo>,
        proof_data: &T,
    ) -> Instruction
    where
        T: Pod + ZkProofData<U>,
        U: Pod,
    {
        let accounts = if let Some(context_state_info) = context_state_info {
            vec![
                AccountMeta::new(*context_state_info.context_state_account, false),
                AccountMeta::new_readonly(*context_state_info.context_state_authority, false),
            ]
        } else {
            vec![]
        };

        let mut data = vec![ToPrimitive::to_u8(self).unwrap()];
        data.extend_from_slice(bytes_of(proof_data));

        Instruction {
            program_id: crate::zk_token_proof_program::id(),
            accounts,
            data,
        }
    }

    pub fn instruction_type(input: &[u8]) -> Option<Self> {
        input
            .first()
            .and_then(|instruction| FromPrimitive::from_u8(*instruction))
    }

    pub fn proof_data<T, U>(input: &[u8]) -> Option<&T>
    where
        T: Pod + ZkProofData<U>,
        U: Pod,
    {
        input
            .get(1..)
            .and_then(|data| bytemuck::try_from_bytes(data).ok())
    }
}
