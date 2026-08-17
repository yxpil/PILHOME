//! VEScaner —— 视频设备发现程序
//!
//! 探测局域网内的视频设备(RTSP / ONVIF),建立视频源台账:
//! - [`probe`] 视频设备探测(RTSP 554 / ONVIF 8000)
//! - [`frame`] 帧采集辅助(灰度帧差运动检测,为审计模块供数)

pub mod frame;
pub mod probe;

pub use frame::{BBox, FrameDiff, MotionReport};
pub use probe::{ProbeConfig, VideoDevice, probe_network};