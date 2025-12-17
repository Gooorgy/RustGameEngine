use assets::MeshHandle;

#[derive(Clone, Copy)]
pub enum EnginePrimitiveType {
    StaticMesh(MeshHandle),
    Unknown,
}
