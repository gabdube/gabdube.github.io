use zerocopy_derive::{FromBytes, Immutable, IntoBytes};
use crate::shared::AABB;

#[derive(Copy, Clone, Default, FromBytes, IntoBytes, Immutable)]
pub struct StaticSprite {
    pub texcoord: AABB,
}
