pub enum CaptureMode {
    Visio,       // ScreenCaptureKit + mic
    InPerson,    // mic only
}

pub struct AudioCapturer {
    mode: CaptureMode,
}

impl AudioCapturer {
    pub fn new(mode: CaptureMode) -> Self {
        Self { mode }
    }
}
