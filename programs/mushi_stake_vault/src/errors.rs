use anchor_lang::prelude::error_code;

#[error_code]
pub enum MushiStakeVaultError {
    #[msg("Insufficient mushi token amount")]
    InsufficientMushiTokenAmount,
    #[msg("Insufficient eclipse token amount")]
    InsufficientEclipseTokenAmount,
    #[msg("Insufficient stake token amount")]
    InsufficientStakeTokenAmount,
    #[msg("Unauthorized program call")]
    UnauthorizedProgramCall,
    #[msg("Only admin can perform this action")]
    UnauthorizedAdminAction,
}