pub fn sp(f:f32) -> f32 { f * 65536.0 }
pub fn inch(f:f32) -> f32 { f * 72.27 }
pub fn cm(f:f32) -> f32 { inch(f) / 2.54 }
pub fn mm(f:f32) -> f32 { cm(f) / 10.0 }