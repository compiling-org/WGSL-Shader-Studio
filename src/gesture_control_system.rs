use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Window, Navigator, MediaDevices, MediaStream, MediaStreamConstraints};

#[derive(Debug, Clone)]
pub struct HandLandmark {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub visibility: f32,
}

#[derive(Debug, Clone)]
pub struct HandTrackingData {
    pub landmarks: Vec<HandLandmark>,
    pub handedness: String, // "Left" or "Right"
    pub confidence: f32,
    pub timestamp: f64,
    pub gesture: HandGesture,
}

#[derive(Debug, Clone)]
pub enum HandGesture {
    OpenPalm,
    ClosedFist,
    Pointing,
    PeaceSign,
    ThumbsUp,
    ThumbsDown,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct LeapMotionData {
    pub hands: Vec<LeapHand>,
    pub gestures: Vec<LeapGesture>,
    pub timestamp: f64,
}

#[derive(Debug, Clone)]
pub struct LeapHand {
    pub id: u32,
    pub palm_position: [f32; 3],
    pub palm_normal: [f32; 3],
    pub palm_direction: [f32; 3],
    pub grab_strength: f32,
    pub pinch_strength: f32,
    pub confidence: f32,
    pub fingers: Vec<LeapFinger>,
}

#[derive(Debug, Clone)]
pub struct LeapFinger {
    pub type_: FingerType,
    pub tip_position: [f32; 3],
    pub direction: [f32; 3],
    pub length: f32,
    pub width: f32,
    pub is_extended: bool,
}

#[derive(Debug, Clone)]
pub enum FingerType {
    Thumb,
    Index,
    Middle,
    Ring,
    Pinky,
}

#[derive(Debug, Clone)]
pub enum LeapGesture {
    Swipe {
        direction: [f32; 3],
        speed: f32,
        duration: f32,
    },
    Circle {
        center: [f32; 3],
        radius: f32,
        progress: f32,
        clock_wise: bool,
    },
    ScreenTap {
        position: [f32; 3],
        direction: [f32; 3],
    },
    KeyTap {
        position: [f32; 3],
        direction: [f32; 3],
    },
}

pub struct MediaPipeIntegration {
    window: Option<Window>,
    is_initialized: Arc<AtomicBool>,
    hand_tracking_data: Arc<Mutex<Vec<HandTrackingData>>>,
    gesture_history: Arc<Mutex<VecDeque<HandGesture>>>,
    video_element: Option<web_sys::HtmlVideoElement>,
    canvas_element: Option<web_sys::HtmlCanvasElement>,
    media_stream: Option<MediaStream>,
}

impl MediaPipeIntegration {
    pub fn new() -> Self {
        Self {
            window: None,
            is_initialized: Arc::new(AtomicBool::new(false)),
            hand_tracking_data: Arc::new(Mutex::new(Vec::new())),
            gesture_history: Arc::new(Mutex::new(VecDeque::with_capacity(100))),
            video_element: None,
            canvas_element: None,
            media_stream: None,
        }
    }

    pub async fn initialize(&mut self) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or("No window available")?;
        self.window = Some(window.clone());

        // Request camera access
        let navigator = window.navigator();
        let media_devices: MediaDevices = navigator.media_devices()?;

        let mut constraints = MediaStreamConstraints::new();
        let video_constraints = js_sys::Object::new();
        js_sys::Reflect::set(&video_constraints, &"width".into(), &1280.into())?;
        js_sys::Reflect::set(&video_constraints, &"height".into(), &720.into())?;
        js_sys::Reflect::set(&video_constraints, &"facingMode".into(), &"user".into())?;
        constraints.video(&video_constraints);

        let promise = media_devices.get_user_media_with_constraints(&constraints)?;
        let media_stream = JsFuture::from(promise).await?;
        let media_stream: MediaStream = media_stream.dyn_into()?;

        // Create video element
        let document = window.document().ok_or("No document available")?;
        let video_element = document.create_element("video")?;
        let video_element: web_sys::HtmlVideoElement = video_element.dyn_into()?;
        video_element.set_src_object(Some(&media_stream));
        video_element.set_autoplay(true);
        video_element.set_muted(true);
        video_element.set_width(1280);
        video_element.set_height(720);

        // Create canvas element for processing
        let canvas_element = document.create_element("canvas")?;
        let canvas_element: web_sys::HtmlCanvasElement = canvas_element.dyn_into()?;
        canvas_element.set_width(1280);
        canvas_element.set_height(720);

        self.video_element = Some(video_element.clone());
        self.canvas_element = Some(canvas_element);
        self.media_stream = Some(media_stream);

        // Initialize MediaPipe Hands
        self.initialize_mediapipe_hands(&video_element).await?;

        self.is_initialized.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn initialize_mediapipe_hands(&self, video_element: &web_sys::HtmlVideoElement) -> Result<(), JsValue> {
        // Load MediaPipe Hands solution
        let script = r#"
            import {Hands} from 'https://cdn.jsdelivr.net/npm/@mediapipe/hands@0.4.1675469240/hands.js';
            import {Camera} from 'https://cdn.jsdelivr.net/npm/@mediapipe/camera@0.3.1640029074/camera.js';
            
            window.mediapipeHands = new Hands({
                locateFile: (file) => {
                    return `https://cdn.jsdelivr.net/npm/@mediapipe/hands@0.4.1675469240/${file}`;
                }
            });
            
            window.mediapipeHands.setOptions({
                maxNumHands: 2,
                modelComplexity: 1,
                minDetectionConfidence: 0.7,
                minTrackingConfidence: 0.5
            });
        "#;

        if let Some(window) = &self.window {
            let _ = window.eval_with_source(script, "mediapipe_hands_init.js")?;
        }

        Ok(())
    }

    pub fn start_hand_tracking(&self) -> Result<(), JsValue> {
        if !self.is_initialized.load(Ordering::SeqCst) {
            return Err(JsValue::from_str("MediaPipe not initialized"));
        }

        let hand_tracking_data = self.hand_tracking_data.clone();
        let gesture_history = self.gesture_history.clone();

        let on_results = Closure::wrap(Box::new(move |results: JsValue| {
            if let Ok(hand_data) = self.process_mediapipe_results(results) {
                if let Ok(mut tracking_data) = hand_tracking_data.lock() {
                    *tracking_data = hand_data.clone();
                }

                for hand in &hand_data {
                    if let Ok(mut history) = gesture_history.lock() {
                        history.push_back(hand.gesture.clone());
                        if history.len() > 100 {
                            history.pop_front();
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(JsValue)>);

        // Set up MediaPipe callback
        if let Some(window) = &self.window {
            let hands_obj = js_sys::Reflect::get(&window, &"mediapipeHands".into())?;
            let _ = js_sys::Reflect::set(&hands_obj, &"onResults".into(), on_results.as_ref())?;
        }

        on_results.forget();
        Ok(())
    }

    fn process_mediapipe_results(&self, results: JsValue) -> Result<Vec<HandTrackingData>, JsValue> {
        let multi_hand_landmarks = js_sys::Reflect::get(&results, &"multiHandLandmarks".into())?;
        let multi_handedness = js_sys::Reflect::get(&results, &"multiHandedness".into())?;

        let mut hand_data = Vec::new();

        if let (Ok(landmarks_array), Ok(handedness_array)) = (
            multi_hand_landmarks.dyn_ref::<js_sys::Array>(),
            multi_handedness.dyn_ref::<js_sys::Array>()
        ) {
            for i in 0..landmarks_array.length() {
                if let (Ok(hand_landmarks), Ok(hand_handedness)) = (
                    landmarks_array.get(i).dyn_into::<js_sys::Array>(),
                    handedness_array.get(i).dyn_into::<js_sys::Object>()
                ) {
                    let landmarks = self.extract_landmarks(&hand_landmarks)?;
                    let handedness = self.extract_handedness(&hand_handedness)?;
                    let confidence = self.calculate_confidence(&landmarks);
                    let gesture = self.classify_gesture(&landmarks);

                    hand_data.push(HandTrackingData {
                        landmarks,
                        handedness,
                        confidence,
                        timestamp: web_sys::window()
                            .and_then(|w| w.performance())
                            .map(|p| p.now())
                            .unwrap_or(0.0),
                        gesture,
                    });
                }
            }
        }

        Ok(hand_data)
    }

    fn extract_landmarks(&self, hand_landmarks: &js_sys::Array) -> Result<Vec<HandLandmark>, JsValue> {
        let mut landmarks = Vec::new();

        for i in 0..hand_landmarks.length() {
            if let Ok(landmark) = hand_landmarks.get(i).dyn_into::<js_sys::Object>() {
                let x = js_sys::Reflect::get(&landmark, &"x".into())?
                    .as_f64().unwrap_or(0.0) as f32;
                let y = js_sys::Reflect::get(&landmark, &"y".into())?
                    .as_f64().unwrap_or(0.0) as f32;
                let z = js_sys::Reflect::get(&landmark, &"z".into())?
                    .as_f64().unwrap_or(0.0) as f32;
                let visibility = js_sys::Reflect::get(&landmark, &"visibility".into())?
                    .as_f64().unwrap_or(0.0) as f32;

                landmarks.push(HandLandmark { x, y, z, visibility });
            }
        }

        Ok(landmarks)
    }

    fn extract_handedness(&self, hand_handedness: &js_sys::Object) -> Result<String, JsValue> {
        let label = js_sys::Reflect::get(hand_handedness, &"label".into())?
            .as_string().unwrap_or("Unknown".to_string());
        Ok(label)
    }

    fn calculate_confidence(&self, landmarks: &[HandLandmark]) -> f32 {
        let total_visibility: f32 = landmarks.iter().map(|l| l.visibility).sum();
        total_visibility / landmarks.len() as f32
    }

    fn classify_gesture(&self, landmarks: &[HandLandmark]) -> HandGesture {
        if landmarks.len() < 21 {
            return HandGesture::Unknown;
        }

        // Simple gesture classification based on finger positions
        let thumb_tip = &landmarks[4];
        let index_tip = &landmarks[8];
        let middle_tip = &landmarks[12];
        let ring_tip = &landmarks[16];
        let pinky_tip = &landmarks[20];

        let wrist = &landmarks[0];

        // Calculate finger distances from wrist
        let thumb_distance = ((thumb_tip.x - wrist.x).powi(2) + (thumb_tip.y - wrist.y).powi(2)).sqrt();
        let index_distance = ((index_tip.x - wrist.x).powi(2) + (index_tip.y - wrist.y).powi(2)).sqrt();
        let middle_distance = ((middle_tip.x - wrist.x).powi(2) + (middle_tip.y - wrist.y).powi(2)).sqrt();
        let ring_distance = ((ring_tip.x - wrist.x).powi(2) + (ring_tip.y - wrist.y).powi(2)).sqrt();
        let pinky_distance = ((pinky_tip.x - wrist.x).powi(2) + (pinky_tip.y - wrist.y).powi(2)).sqrt();

        let avg_distance = (index_distance + middle_distance + ring_distance + pinky_distance) / 4.0;

        // Open palm: all fingers extended
        if thumb_distance > avg_distance * 0.8 &&
           index_distance > avg_distance * 0.8 &&
           middle_distance > avg_distance * 0.8 &&
           ring_distance > avg_distance * 0.8 &&
           pinky_distance > avg_distance * 0.8 {
            return HandGesture::OpenPalm;
        }

        // Closed fist: all fingers close to wrist
        if thumb_distance < avg_distance * 0.4 &&
           index_distance < avg_distance * 0.4 &&
           middle_distance < avg_distance * 0.4 &&
           ring_distance < avg_distance * 0.4 &&
           pinky_distance < avg_distance * 0.4 {
            return HandGesture::ClosedFist;
        }

        // Pointing: only index finger extended
        if index_distance > avg_distance * 0.8 &&
           middle_distance < avg_distance * 0.5 &&
           ring_distance < avg_distance * 0.5 &&
           pinky_distance < avg_distance * 0.5 {
            return HandGesture::Pointing;
        }

        // Peace sign: index and middle fingers extended
        if index_distance > avg_distance * 0.8 &&
           middle_distance > avg_distance * 0.8 &&
           ring_distance < avg_distance * 0.5 &&
           pinky_distance < avg_distance * 0.5 {
            return HandGesture::PeaceSign;
        }

        HandGesture::Unknown
    }

    pub fn get_current_hand_data(&self) -> Vec<HandTrackingData> {
        self.hand_tracking_data.lock().unwrap().clone()
    }

    pub fn get_gesture_history(&self) -> Vec<HandGesture> {
        self.gesture_history.lock().unwrap().iter().cloned().collect()
    }

    pub fn is_initialized(&self) -> bool {
        self.is_initialized.load(Ordering::SeqCst)
    }
}

pub struct LeapMotionIntegration {
    is_connected: Arc<AtomicBool>,
    leap_data: Arc<Mutex<LeapMotionData>>,
    websocket: Option<web_sys::WebSocket>,
    connection_id: Arc<AtomicU32>,
}

impl LeapMotionIntegration {
    pub fn new() -> Self {
        Self {
            is_connected: Arc::new(AtomicBool::new(false)),
            leap_data: Arc::new(Mutex::new(LeapMotionData {
                hands: Vec::new(),
                gestures: Vec::new(),
                timestamp: 0.0,
            })),
            websocket: None,
            connection_id: Arc::new(AtomicU32::new(0)),
        }
    }

    pub async fn connect(&mut self, host: &str, port: u16) -> Result<(), JsValue> {
        let ws_url = format!("ws://{}:{}/v6.json", host, port);
        let websocket = web_sys::WebSocket::new(&ws_url)?;

        let is_connected = self.is_connected.clone();
        let leap_data = self.leap_data.clone();
        let connection_id = self.connection_id.clone();

        // Set up WebSocket event handlers
        let on_open = Closure::wrap(Box::new(move |_event: JsValue| {
            is_connected.store(true, Ordering::SeqCst);
            web_sys::console::log_1(&"LeapMotion WebSocket connected".into());
        }) as Box<dyn FnMut(JsValue)>);

        let on_message = Closure::wrap(Box::new(move |event: JsValue| {
            if let Ok(message_event) = event.dyn_into::<web_sys::MessageEvent>() {
                if let Ok(data) = message_event.data().dyn_into::<js_sys::JsString>() {
                    let json_str: String = data.into();
                    if let Ok(leap_frame) = serde_json::from_str::<serde_json::Value>(&json_str) {
                        self.process_leap_frame(leap_frame, &leap_data);
                    }
                }
            }
        }) as Box<dyn FnMut(JsValue)>);

        let on_close = Closure::wrap(Box::new(move |_event: JsValue| {
            is_connected.store(false, Ordering::SeqCst);
            web_sys::console::log_1(&"LeapMotion WebSocket disconnected".into());
        }) as Box<dyn FnMut(JsValue)>);

        let on_error = Closure::wrap(Box::new(move |event: JsValue| {
            web_sys::console::error_1(&format!("LeapMotion WebSocket error: {:?}", event).into());
        }) as Box<dyn FnMut(JsValue)>);

        websocket.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        websocket.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        websocket.set_onclose(Some(on_close.as_ref().unchecked_ref()));
        websocket.set_onerror(Some(on_error.as_ref().unchecked_ref()));

        // Send initial configuration
        let config = serde_json::json!({
            "enableGestures": true,
            "background": false,
            "optimizeHMD": false
        });

        websocket.send_with_str(&config.to_string())?;

        self.websocket = Some(websocket);
        connection_id.fetch_add(1, Ordering::SeqCst);

        on_open.forget();
        on_message.forget();
        on_close.forget();
        on_error.forget();

        Ok(())
    }

    fn process_leap_frame(&self, frame: serde_json::Value, leap_data: &Arc<Mutex<LeapMotionData>>) {
        let mut hands = Vec::new();
        let mut gestures = Vec::new();

        // Process hands
        if let Some(hands_array) = frame["hands"].as_array() {
            for hand_data in hands_array {
                if let Ok(hand) = self.parse_leap_hand(hand_data) {
                    hands.push(hand);
                }
            }
        }

        // Process gestures
        if let Some(gestures_array) = frame["gestures"].as_array() {
            for gesture_data in gestures_array {
                if let Ok(gesture) = self.parse_leap_gesture(gesture_data) {
                    gestures.push(gesture);
                }
            }
        }

        let timestamp = frame["timestamp"]
            .as_f64()
            .or_else(|| frame["currentFrameRate"].as_f64())
            .unwrap_or(0.0);

        if let Ok(mut data) = leap_data.lock() {
            data.hands = hands;
            data.gestures = gestures;
            data.timestamp = timestamp;
        }
    }

    fn parse_leap_hand(&self, hand_data: &serde_json::Value) -> Result<LeapHand, JsValue> {
        let id = hand_data["id"].as_u64().unwrap_or(0) as u32;
        
        let palm_position = self.parse_vector3(&hand_data["palmPosition"])?;
        let palm_normal = self.parse_vector3(&hand_data["palmNormal"])?;
        let palm_direction = self.parse_vector3(&hand_data["direction"])?;
        
        let grab_strength = hand_data["grabStrength"].as_f64().unwrap_or(0.0) as f32;
        let pinch_strength = hand_data["pinchStrength"].as_f64().unwrap_or(0.0) as f32;
        let confidence = hand_data["confidence"].as_f64().unwrap_or(0.0) as f32;

        let mut fingers = Vec::new();
        if let Some(fingers_array) = hand_data["fingers"].as_array() {
            for finger_data in fingers_array {
                if let Ok(finger) = self.parse_leap_finger(finger_data) {
                    fingers.push(finger);
                }
            }
        }

        Ok(LeapHand {
            id,
            palm_position,
            palm_normal,
            palm_direction,
            grab_strength,
            pinch_strength,
            confidence,
            fingers,
        })
    }

    fn parse_leap_finger(&self, finger_data: &serde_json::Value) -> Result<LeapFinger, JsValue> {
        let type_str = finger_data["type"].as_str().unwrap_or("unknown");
        let type_ = match type_str {
            "thumb" => FingerType::Thumb,
            "index" => FingerType::Index,
            "middle" => FingerType::Middle,
            "ring" => FingerType::Ring,
            "pinky" => FingerType::Pinky,
            _ => FingerType::Index,
        };

        let tip_position = self.parse_vector3(&finger_data["tipPosition"])?;
        let direction = self.parse_vector3(&finger_data["direction"])?;
        let length = finger_data["length"].as_f64().unwrap_or(0.0) as f32;
        let width = finger_data["width"].as_f64().unwrap_or(0.0) as f32;
        let is_extended = finger_data["extended"].as_bool().unwrap_or(false);

        Ok(LeapFinger {
            type_,
            tip_position,
            direction,
            length,
            width,
            is_extended,
        })
    }

    fn parse_leap_gesture(&self, gesture_data: &serde_json::Value) -> Result<LeapGesture, JsValue> {
        let gesture_type = gesture_data["type"].as_str().unwrap_or("unknown");
        
        match gesture_type {
            "swipe" => {
                let direction = self.parse_vector3(&gesture_data["direction"])?;
                let speed = gesture_data["speed"].as_f64().unwrap_or(0.0) as f32;
                let duration = gesture_data["duration"].as_f64().unwrap_or(0.0) as f32;
                
                Ok(LeapGesture::Swipe {
                    direction,
                    speed,
                    duration,
                })
            },
            "circle" => {
                let center = self.parse_vector3(&gesture_data["center"])?;
                let radius = gesture_data["radius"].as_f64().unwrap_or(0.0) as f32;
                let progress = gesture_data["progress"].as_f64().unwrap_or(0.0) as f32;
                let clock_wise = gesture_data["clockwise"].as_bool().unwrap_or(true);
                
                Ok(LeapGesture::Circle {
                    center,
                    radius,
                    progress,
                    clock_wise,
                })
            },
            "screenTap" => {
                let position = self.parse_vector3(&gesture_data["position"])?;
                let direction = self.parse_vector3(&gesture_data["direction"])?;
                
                Ok(LeapGesture::ScreenTap {
                    position,
                    direction,
                })
            },
            "keyTap" => {
                let position = self.parse_vector3(&gesture_data["position"])?;
                let direction = self.parse_vector3(&gesture_data["direction"])?;
                
                Ok(LeapGesture::KeyTap {
                    position,
                    direction,
                })
            },
            _ => Err(JsValue::from_str("Unknown gesture type")),
        }
    }

    fn parse_vector3(&self, vector_data: &serde_json::Value) -> Result<[f32; 3], JsValue> {
        if let Some(array) = vector_data.as_array() {
            if array.len() >= 3 {
                let x = array[0].as_f64().unwrap_or(0.0) as f32;
                let y = array[1].as_f64().unwrap_or(0.0) as f32;
                let z = array[2].as_f64().unwrap_or(0.0) as f32;
                Ok([x, y, z])
            } else {
                Err(JsValue::from_str("Invalid vector3 data"))
            }
        } else {
            Err(JsValue::from_str("Vector3 data is not an array"))
        }
    }

    pub fn get_current_leap_data(&self) -> LeapMotionData {
        self.leap_data.lock().unwrap().clone()
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected.load(Ordering::SeqCst)
    }

    pub fn disconnect(&mut self) {
        if let Some(websocket) = &self.websocket {
            let _ = websocket.close();
        }
        self.websocket = None;
        self.is_connected.store(false, Ordering::SeqCst);
    }
}

pub struct UnifiedGestureSystem {
    mediapipe: MediaPipeIntegration,
    leapmotion: LeapMotionIntegration,
    is_mediapipe_active: Arc<AtomicBool>,
    is_leapmotion_active: Arc<AtomicBool>,
    unified_gestures: Arc<Mutex<Vec<UnifiedGesture>>>,
}

#[derive(Debug, Clone)]
pub struct UnifiedGesture {
    pub gesture_type: UnifiedGestureType,
    pub confidence: f32,
    pub position: [f32; 3],
    pub timestamp: f64,
    pub source: GestureSource,
}

#[derive(Debug, Clone)]
pub enum UnifiedGestureType {
    Point(f32, f32), // x, y coordinates normalized
    Grab(f32), // strength 0-1
    Swipe(f32, f32, f32), // direction x, y, speed
    Circle(f32), // progress 0-1
    Pinch(f32), // strength 0-1
    Wave(f32), // frequency
}

#[derive(Debug, Clone)]
pub enum GestureSource {
    MediaPipe,
    LeapMotion,
}

impl UnifiedGestureSystem {
    pub fn new() -> Self {
        Self {
            mediapipe: MediaPipeIntegration::new(),
            leapmotion: LeapMotionIntegration::new(),
            is_mediapipe_active: Arc::new(AtomicBool::new(false)),
            is_leapmotion_active: Arc::new(AtomicBool::new(false)),
            unified_gestures: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn initialize_mediapipe(&mut self) -> Result<(), JsValue> {
        self.mediapipe.initialize().await?;
        self.is_mediapipe_active.store(true, Ordering::SeqCst);
        Ok(())
    }

    pub async fn connect_leapmotion(&mut self, host: &str, port: u16) -> Result<(), JsValue> {
        self.leapmotion.connect(host, port).await?;
        self.is_leapmotion_active.store(true, Ordering::SeqCst);
        Ok(())
    }

    pub fn start_gesture_processing(&self) {
        if self.is_mediapipe_active.load(Ordering::SeqCst) {
            let _ = self.mediapipe.start_hand_tracking();
        }

        // Start unified gesture processing loop
        self.start_unified_processing();
    }

    fn start_unified_processing(&self) {
        let unified_gestures = self.unified_gestures.clone();
        let mediapipe_data = self.mediapipe.get_current_hand_data();
        let leapmotion_data = self.leapmotion.get_current_leap_data();
        let is_mediapipe_active = self.is_mediapipe_active.clone();
        let is_leapmotion_active = self.is_leapmotion_active.clone();

        let process_gestures = move || {
            let mut gestures = Vec::new();

            // Process MediaPipe data
            if is_mediapipe_active.load(Ordering::SeqCst) {
                for hand in &mediapipe_data {
                    if let Some(unified) = self.unify_mediapipe_gesture(hand) {
                        gestures.push(unified);
                    }
                }
            }

            // Process LeapMotion data
            if is_leapmotion_active.load(Ordering::SeqCst) {
                for hand in &leapmotion_data.hands {
                    if let Some(unified) = self.unify_leapmotion_gesture(hand) {
                        gestures.push(unified);
                    }
                }

                for leap_gesture in &leapmotion_data.gestures {
                    if let Some(unified) = self.unify_leapmotion_specific_gesture(leap_gesture) {
                        gestures.push(unified);
                    }
                }
            }

            if let Ok(mut unified_data) = unified_gestures.lock() {
                *unified_data = gestures;
            }
        };

        let process_closure = Closure::wrap(Box::new(process_gestures) as Box<dyn FnMut()>);

        if let Some(window) = web_sys::window() {
            let process_fn = process_closure.as_ref().unchecked_ref();
            let _ = window.set_interval_with_callback_and_timeout_and_arguments_0(
                process_fn,
                16, // ~60fps
            );
        }

        process_closure.forget();
    }

    fn unify_mediapipe_gesture(&self, hand: &HandTrackingData) -> Option<UnifiedGesture> {
        if hand.landmarks.is_empty() {
            return None;
        }

        let palm = &hand.landmarks[0]; // Wrist landmark
        let normalized_x = palm.x;
        let normalized_y = palm.y;

        match &hand.gesture {
            HandGesture::OpenPalm => Some(UnifiedGesture {
                gesture_type: UnifiedGestureType::Grab(0.0),
                confidence: hand.confidence,
                position: [normalized_x, normalized_y, palm.z],
                timestamp: hand.timestamp,
                source: GestureSource::MediaPipe,
            }),
            HandGesture::ClosedFist => Some(UnifiedGesture {
                gesture_type: UnifiedGestureType::Grab(1.0),
                confidence: hand.confidence,
                position: [normalized_x, normalized_y, palm.z],
                timestamp: hand.timestamp,
                source: GestureSource::MediaPipe,
            }),
            HandGesture::Pointing => Some(UnifiedGesture {
                gesture_type: UnifiedGestureType::Point(normalized_x, normalized_y),
                confidence: hand.confidence,
                position: [normalized_x, normalized_y, palm.z],
                timestamp: hand.timestamp,
                source: GestureSource::MediaPipe,
            }),
            _ => None,
        }
    }

    fn unify_leapmotion_gesture(&self, hand: &LeapHand) -> Option<UnifiedGesture> {
        let [x, y, z] = hand.palm_position;
        let normalized_x = (x + 200.0) / 400.0; // Normalize from -200 to 200 range
        let normalized_y = (y + 200.0) / 400.0; // Normalize from -200 to 200 range

        Some(UnifiedGesture {
            gesture_type: UnifiedGestureType::Grab(hand.grab_strength),
            confidence: hand.confidence,
            position: [normalized_x, normalized_y, z],
            timestamp: web_sys::window()
                .and_then(|w| w.performance())
                .map(|p| p.now())
                .unwrap_or(0.0),
            source: GestureSource::LeapMotion,
        })
    }

    fn unify_leapmotion_specific_gesture(&self, gesture: &LeapGesture) -> Option<UnifiedGesture> {
        match gesture {
            LeapGesture::Swipe { direction, speed, .. } => Some(UnifiedGesture {
                gesture_type: UnifiedGestureType::Swipe(direction[0], direction[1], *speed),
                confidence: 0.9,
                position: [0.0, 0.0, 0.0],
                timestamp: web_sys::window()
                    .and_then(|w| w.performance())
                    .map(|p| p.now())
                    .unwrap_or(0.0),
                source: GestureSource::LeapMotion,
            }),
            LeapGesture::Circle { progress, .. } => Some(UnifiedGesture {
                gesture_type: UnifiedGestureType::Circle(*progress),
                confidence: 0.9,
                position: [0.0, 0.0, 0.0],
                timestamp: web_sys::window()
                    .and_then(|w| w.performance())
                    .map(|p| p.now())
                    .unwrap_or(0.0),
                source: GestureSource::LeapMotion,
            }),
            _ => None,
        }
    }

    pub fn get_current_gestures(&self) -> Vec<UnifiedGesture> {
        self.unified_gestures.lock().unwrap().clone()
    }

    pub fn get_shader_uniforms(&self) -> GestureShaderUniforms {
        let gestures = self.get_current_gestures();
        
        let mut point_x = 0.0;
        let mut point_y = 0.0;
        let mut grab_strength = 0.0;
        let mut swipe_x = 0.0;
        let mut swipe_y = 0.0;
        let mut swipe_speed = 0.0;
        let mut circle_progress = 0.0;
        let mut pinch_strength = 0.0;
        let mut wave_frequency = 0.0;
        let mut gesture_count = 0.0;

        for gesture in &gestures {
            match &gesture.gesture_type {
                UnifiedGestureType::Point(x, y) => {
                    point_x = *x;
                    point_y = *y;
                },
                UnifiedGestureType::Grab(strength) => {
                    grab_strength = *strength;
                },
                UnifiedGestureType::Swipe(x, y, speed) => {
                    swipe_x = *x;
                    swipe_y = *y;
                    swipe_speed = *speed;
                },
                UnifiedGestureType::Circle(progress) => {
                    circle_progress = *progress;
                },
                UnifiedGestureType::Pinch(strength) => {
                    pinch_strength = *strength;
                },
                UnifiedGestureType::Wave(frequency) => {
                    wave_frequency = *frequency;
                },
            }
        }

        gesture_count = gestures.len() as f32;

        GestureShaderUniforms {
            u_gesture_point: [point_x, point_y],
            u_gesture_grab: grab_strength,
            u_gesture_swipe: [swipe_x, swipe_y, swipe_speed],
            u_gesture_circle: circle_progress,
            u_gesture_pinch: pinch_strength,
            u_gesture_wave: wave_frequency,
            u_gesture_count: gesture_count,
            u_gesture_active: if gesture_count > 0.0 { 1.0 } else { 0.0 },
        }
    }

    pub fn is_mediapipe_active(&self) -> bool {
        self.is_mediapipe_active.load(Ordering::SeqCst)
    }

    pub fn is_leapmotion_active(&self) -> bool {
        self.is_leapmotion_active.load(Ordering::SeqCst)
    }
}

#[derive(Debug, Clone)]
pub struct GestureShaderUniforms {
    pub u_gesture_point: [f32; 2],
    pub u_gesture_grab: f32,
    pub u_gesture_swipe: [f32; 3],
    pub u_gesture_circle: f32,
    pub u_gesture_pinch: f32,
    pub u_gesture_wave: f32,
    pub u_gesture_count: f32,
    pub u_gesture_active: f32,
}

impl GestureShaderUniforms {
    pub fn to_wgsl_uniforms(&self) -> String {
        r#"
@group(2) @binding(0) var<uniform> gesture_point: vec2<f32>;
@group(2) @binding(1) var<uniform> gesture_grab: f32;
@group(2) @binding(2) var<uniform> gesture_swipe: vec3<f32>;
@group(2) @binding(3) var<uniform> gesture_circle: f32;
@group(2) @binding(4) var<uniform> gesture_pinch: f32;
@group(2) @binding(5) var<uniform> gesture_wave: f32;
@group(2) @binding(6) var<uniform> gesture_count: f32;
@group(2) @binding(7) var<uniform> gesture_active: f32;
"#.to_string()
    }
}