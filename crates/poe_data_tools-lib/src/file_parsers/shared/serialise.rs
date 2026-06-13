use serde::Serializer;
use serde_with::SerializeAs;

/// For serlializing f16 types since serde doesn't implement it natively
pub struct SerF16;

impl SerializeAs<f16> for SerF16 {
    fn serialize_as<S>(source: &f16, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f32(*source as f32)
    }
}
