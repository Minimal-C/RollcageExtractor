const MODL_MAGIC: &[u8; 4] = &[0x4D, 0x4F, 0x44, 0x4C]; // "MODL"

#[derive(Debug, Clone, Copy)]
pub struct ModlHeader {
    pub signature: [u8; MODL_MAGIC.len()],
    pub num_veritices: u16,
    pub num_faces: u16,
    pub unknown_1: u32,
    pub offset_to_face_data: u32, // from current modl
    pub offset_to_unknown_data_1: u32, // from current modl
    pub offset_to_face_data_index: u32, // from modl section start
    pub offset_to_unknown_data_2: u32, // from current modl
    pub offset_to_sub_model_1: u32,
    pub offset_to_sub_model_2: u32,
    pub unknown_2: [u8;36],
    pub offset_to_unknown_data_3: u32 // from current modl
}