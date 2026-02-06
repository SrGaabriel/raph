pub mod ctx;

use crate::{elaboration::ctx::{LocalContext, MetavarContext}, module::unique::UniqueGen};

pub struct ElabState {
    gen_: UniqueGen,
    mctx: MetavarContext,
    lctx: LocalContext,
}