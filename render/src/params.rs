use serde::Deserialize;

#[derive(Deserialize)]
pub struct FractalParams {
    #[serde(rename = "nT")]
    pub n_transforms: usize,
    #[serde(rename = "numDeg")]
    pub num_deg: usize,
    #[serde(rename = "denDeg")]
    pub den_deg: usize,
    pub normalize: bool,
    #[serde(rename = "colorWeight")]
    pub color_weight: f32,
    #[serde(rename = "thresholdPct")]
    pub threshold_pct: u32,
    #[serde(rename = "camAz", default)]
    pub cam_az: f32,
    #[serde(rename = "camEl", default)]
    pub cam_el: f32,
    #[serde(rename = "camDist", default = "default_cam_dist")]
    pub cam_dist: f32,
    pub ang: Vec<f32>,
}

fn default_cam_dist() -> f32 {
    2.2
}

impl FractalParams {
    pub fn load(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let data = std::fs::read_to_string(path)?;
        let params: Self = serde_json::from_str(&data)?;
        if params.ang.len() != 144 {
            return Err(format!("Expected 144 coefficients, got {}", params.ang.len()).into());
        }
        if params.n_transforms == 0 || params.n_transforms > 6 {
            return Err(format!("nT must be 1-6, got {}", params.n_transforms).into());
        }
        Ok(params)
    }
}
