use std::collections::HashSet;

use once_cell::sync::Lazy;

pub const IDENT_INC_DP: u8 = b'>';
pub const IDENT_DEC_DP: u8 = b'<';
pub const IDENT_INC_DATA: u8 = b'+';
pub const IDENT_DEC_DATA: u8 = b'-';
pub const IDENT_WRITE_BYTE: u8 = b'.';
pub const IDENT_READ_BYTE: u8 = b',';
pub const IDENT_JUMP_ZERO: u8 = b'[';
pub const IDENT_JUMP_NOT_ZERO: u8 = b']';

pub static IDENTS: Lazy<HashSet<u8>> = Lazy::new(|| {
    let mut idents = HashSet::new();

    idents.insert(IDENT_INC_DP);
    idents.insert(IDENT_DEC_DP);
    idents.insert(IDENT_INC_DATA);
    idents.insert(IDENT_DEC_DATA);
    idents.insert(IDENT_WRITE_BYTE);
    idents.insert(IDENT_READ_BYTE);
    idents.insert(IDENT_JUMP_ZERO);
    idents.insert(IDENT_JUMP_NOT_ZERO);

    idents
});
