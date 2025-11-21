use crate::{Float, Pixel, Point, Vec2};
use bitflags::bitflags;
use smol_str::SmolStr;

pub const POINTS_PER_SCROLL_LINE: Float<Point> = Float::new(40.0);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct ModifiersState {
    control: KeyLocations,
    shift: KeyLocations,
    alt: KeyLocations,
    alt_graph: KeyLocations,
    meta: KeyLocations,
}

impl ModifiersState {
    #[inline]
    fn insert_control(&mut self, location: KeyLocation) {
        self.control.insert(location.into());
    }

    #[inline]
    fn remove_control(&mut self, location: KeyLocation) {
        self.control.remove(location.into());
    }

    #[inline]
    fn insert_shift(&mut self, location: KeyLocation) {
        self.shift.insert(location.into());
    }

    #[inline]
    fn remove_shift(&mut self, location: KeyLocation) {
        self.shift.remove(location.into());
    }

    #[inline]
    fn insert_alt(&mut self, location: KeyLocation) {
        self.alt.insert(location.into());
    }

    #[inline]
    fn remove_alt(&mut self, location: KeyLocation) {
        self.alt.remove(location.into());
    }

    #[inline]
    fn insert_alt_graph(&mut self, location: KeyLocation) {
        self.alt_graph.insert(location.into());
    }

    #[inline]
    fn remove_alt_graph(&mut self, location: KeyLocation) {
        self.alt_graph.remove(location.into());
    }

    #[inline]
    fn insert_meta(&mut self, location: KeyLocation) {
        self.meta.insert(location.into());
    }

    #[inline]
    fn remove_meta(&mut self, location: KeyLocation) {
        self.meta.remove(location.into());
    }
}

bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Modifiers: u8 {
        const CONTROL = 0x01;
        const SHIFT = 0x02;
        const ALT = 0x04;
        const ALT_GRAPH = 0x08;
        const META = 0x10;
    }
}

impl Modifiers {
    /// Checks whether `self` contains pattern and the state of `CONTROL` is exactly equal.
    pub fn matches(self, pattern: Self) -> bool {
        self.contains(pattern)
            && (self.contains(Modifiers::CONTROL) == pattern.contains(Modifiers::CONTROL))
    }
}

impl From<ModifiersState> for Modifiers {
    fn from(state: ModifiersState) -> Self {
        let mut modifiers = Modifiers::empty();
        if !state.control.is_empty() {
            modifiers |= Modifiers::CONTROL;
        }
        if !state.shift.is_empty() {
            modifiers |= Modifiers::SHIFT;
        }
        if !state.alt.is_empty() {
            modifiers |= Modifiers::ALT;
        }
        if !state.alt_graph.is_empty() {
            modifiers |= Modifiers::ALT_GRAPH;
        }
        if !state.meta.is_empty() {
            modifiers |= Modifiers::META;
        }
        modifiers
    }
}

/// Named keys as defined in https://w3c.github.io/uievents-key/
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum NamedKey {
    /// The <kbd>Alt</kbd> (Alternative) key.
    ///
    /// This key enables the alternate modifier function for interpreting concurrent or subsequent keyboard input.<br/>
    /// This key value is also used for the Apple <kbd>Option</kbd> key.
    Alt,
    /// The Alternate Graphics (<kbd>AltGr</kbd> or <kbd>AltGraph</kbd>) key.
    /// This key is used enable the ISO Level 3 shift modifier (the standard <kbd>Shift</kbd> key is the level 2 modifier).
    /// See [ISO9995-1].
    AltGraph,
    /// The <kbd>Caps Lock</kbd> (Capital) key.
    /// Toggle capital character lock function for interpreting subsequent keyboard input event.
    CapsLock,
    /// The <kbd>Control</kbd> or <kbd>Ctrl</kbd> key, to enable control modifier function for interpreting concurrent or subsequent keyboard input.
    Control,
    /// The Function switch <kbd>Fn</kbd> key.
    ///
    /// Activating this key simultaneously with another key changes that key’s value to an alternate character or function.
    /// This key is often handled directly in the keyboard hardware and does not usually generate key events.
    Fn,
    /// The Function-Lock (<kbd>FnLock</kbd> or <kbd>F-Lock</kbd>) key.
    /// Activating this key switches the mode of the keyboard to changes some keys' values to an alternate character or function.
    /// This key is often handled directly in the keyboard hardware and does not usually generate key events.
    FnLock,
    /// The <kbd>Meta</kbd> key, to enable meta modifier function for interpreting concurrent or subsequent keyboard input.
    ///
    /// This key value is used for the <q>Windows Logo</q> key and the Apple <kbd>Command</kbd> or <kbd>⌘</kbd> key.<br/>
    /// In Linux (XKB) terminology, this is often referred to as "Super".
    Meta,
    /// The <kbd>NumLock</kbd> or Number Lock key, to toggle numpad mode function for interpreting subsequent keyboard input.
    NumLock,
    /// The <kbd>Scroll Lock</kbd> key, to toggle between scrolling and cursor movement modes.
    ScrollLock,
    /// The <kbd>Shift</kbd> key, to enable shift modifier function for interpreting concurrent or subsequent keyboard input.
    Shift,
    /// The Symbol modifier key (used on some virtual keyboards).
    Symbol,
    /// The Symbol Lock key.
    SymbolLock,
    /// The <kbd>Enter</kbd> or <kbd>↵</kbd> key, to activate current selection or accept current input.
    ///
    /// This key value is also used for the <kbd>Return</kbd> (Macintosh numpad) key.<br/>
    /// This key value is also used for the Android <code>KEYCODE_DPAD_CENTER</code>.
    #[doc(alias = "Return")]
    Enter,
    /// The Horizontal Tabulation <kbd>Tab</kbd> key.
    Tab,
    /// The down arrow key, to navigate or traverse downward. (<code>KEYCODE_DPAD_DOWN</code>)
    ArrowDown,
    /// The left arrow key, to navigate or traverse leftward. (<code>KEYCODE_DPAD_LEFT</code>)
    ArrowLeft,
    /// The right arrow key, to navigate or traverse rightward. (<code>KEYCODE_DPAD_RIGHT</code>)
    ArrowRight,
    /// The up arrow key, to navigate or traverse upward. (<code>KEYCODE_DPAD_UP</code>)
    ArrowUp,
    /// The End key, used with keyboard entry to go to the end of content (<code>KEYCODE_MOVE_END</code>).
    End,
    /// The Home key, used with keyboard entry, to go to start of content (<code>KEYCODE_MOVE_HOME</code>).
    ///
    /// For the mobile phone <kbd>Home</kbd> key (which goes to the phone’s main screen), use [`GoHome`][NamedKey::GoHome].
    Home,
    /// The Page Down key, to scroll down or display next page of content.
    PageDown,
    /// The Page Up key, to scroll up or display previous page of content.
    PageUp,
    /// The Backspace key. This key value is also used for the key labeled <kbd>Delete</kbd> on MacOS keyboards.
    Backspace,
    /// Remove the currently selected input.
    Clear,
    /// Copy the current selection. (<code>APPCOMMAND_COPY</code>)
    Copy,
    /// The Cursor Select (Crsel) key.
    CrSel,
    /// Cut the current selection. (<code>APPCOMMAND_CUT</code>)
    Cut,
    /// The Delete (Del) Key.
    /// This key value is also used for the key labeled <kbd>Delete</kbd> on MacOS keyboards when modified by the <kbd>Fn</kbd> key.
    Delete,
    /// The Erase to End of Field key.
    /// This key deletes all characters from the current cursor position to the end of the current field.
    EraseEof,
    /// The Extend Selection (Exsel) key.
    ExSel,
    /// The Insert (Ins) key, to toggle between text modes for insertion or overtyping. (<code>KEYCODE_INSERT</code>)
    Insert,
    /// The Paste key. (<code>APPCOMMAND_PASTE</code>)
    Paste,
    /// Redo the last action. (<code>APPCOMMAND_REDO</code>)
    Redo,
    /// Undo the last action. (<code>APPCOMMAND_UNDO</code>)
    Undo,
    /// The Accept (Commit, OK) key. Accept current option or input method sequence conversion.
    Accept,
    /// The Again key, to redo or repeat an action.
    Again,
    /// The Attention (Attn) key.
    Attn,
    /// The Cancel key.
    Cancel,
    /// Show the application’s context menu.
    /// This key is commonly found between the right <kbd>Meta</kbd> key and the right <kbd>Control</kbd> key.
    ContextMenu,
    /// The <kbd>Esc</kbd> key. This key was originally used to initiate an escape sequence, but is
    /// now more generally used to exit or "escape" the current context, such as closing a dialog
    /// or exiting full screen mode.
    Escape,
    /// The Execute key.
    Execute,
    /// Open the Find dialog. (<code>APPCOMMAND_FIND</code>)
    Find,
    /// Open a help dialog or toggle display of help information. (<code>APPCOMMAND_HELP</code>, <code>KEYCODE_HELP</code>)
    Help,
    /// Pause the current state or application (as appropriate).
    ///
    /// Do not use this value for the <kbd>Pause</kbd> button on media controllers. Use [`MediaPause`][NamedKey::MediaPause] instead.
    Pause,
    /// Play or resume the current state or application (as appropriate).
    ///
    /// Do not use this value for the <kbd>Play</kbd> button on media controllers. Use [`MediaPlay`][NamedKey::MediaPlay] instead.
    Play,
    /// The properties (Props) key.
    Props,
    /// The Select key.
    Select,
    /// The ZoomIn key. (<code>KEYCODE_ZOOM_IN</code>)
    ZoomIn,
    /// The ZoomOut key. (<code>KEYCODE_ZOOM_OUT</code>)
    ZoomOut,
    /// The Brightness Down key. Typically controls the display brightness. (<code>KEYCODE_BRIGHTNESS_DOWN</code>)
    BrightnessDown,
    /// The Brightness Up key. Typically controls the display brightness. (<code>KEYCODE_BRIGHTNESS_UP</code>)
    BrightnessUp,
    /// Toggle removable media to eject (open) and insert (close) state. (<code>KEYCODE_MEDIA_EJECT</code>)
    Eject,
    /// The LogOff key.
    LogOff,
    /// Toggle power state. (<code>KEYCODE_POWER</code>)
    ///
    /// Note: Some devices might not expose this key to the operating environment.
    Power,
    /// The <kbd>PowerOff</kbd> key. Sometime called <kbd>PowerDown</kbd>.
    PowerOff,
    /// The <kbd>Print Screen</kbd> or <kbd>SnapShot</kbd> key, to initiate print-screen function.
    PrintScreen,
    /// The Hibernate key.
    /// This key saves the current state of the computer to disk so that it can be restored. The computer will then shutdown.
    Hibernate,
    /// The Standby key.
    ///
    /// This key turns off the display and places the computer into a low-power mode without completely shutting down.<br/>
    /// It is sometimes labelled <kbd>Suspend</kbd> or <kbd>Sleep</kbd> key. (<code>KEYCODE_SLEEP</code>)
    Standby,
    /// The WakeUp key. (<code>KEYCODE_WAKEUP</code>)
    WakeUp,
    /// The All Candidates key, to initiate the multi-candidate mode.
    AllCandidates,
    /// The Alphanumeric key.
    Alphanumeric,
    /// The Code Input key, to initiate the Code Input mode to allow characters to be entered by their code points.
    CodeInput,
    /// The Compose key, also known as <em>Multi_key</em> on the X Window System.
    ///
    /// This key acts in a manner similar to a dead key, triggering a mode where subsequent key presses are combined to produce a different character.
    Compose,
    /// The Convert key, to convert the current input method sequence.
    Convert,
    /// The Final Mode <kbd>Final</kbd> key used on some Asian keyboards, to enable the final mode for IMEs.
    FinalMode,
    /// Switch to the first character group. (ISO/IEC 9995)
    GroupFirst,
    /// Switch to the last character group. (ISO/IEC 9995)
    GroupLast,
    /// Switch to the next character group. (ISO/IEC 9995)
    GroupNext,
    /// Switch to the previous character group. (ISO/IEC 9995)
    GroupPrevious,
    /// The Mode Change key, to toggle between or cycle through input modes of IMEs.
    ModeChange,
    /// The Next Candidate function key.
    NextCandidate,
    /// The NonConvert ("Don’t Convert") key, to accept current input method sequence without conversion in IMEs.
    NonConvert,
    /// The Previous Candidate function key.
    PreviousCandidate,
    /// The Process key.
    Process,
    /// The Single Candidate function key.
    SingleCandidate,
    /// The Hangul (Korean characters) Mode key, to toggle between Hangul and English modes.
    HangulMode,
    /// The Hanja (Korean characters) Mode key.
    HanjaMode,
    /// The Junja (Korean characters) Mode key.
    JunjaMode,
    /// The Eisu key.
    ///
    /// This key may close the IME, but its purpose is defined by the current IME. (<code>KEYCODE_EISU</code>)
    Eisu,
    /// The (Half-Width) Characters key.
    Hankaku,
    /// The Hiragana (Japanese Kana characters) key.
    Hiragana,
    /// The Hiragana/Katakana toggle key. (<code>KEYCODE_KATAKANA_HIRAGANA</code>)
    HiraganaKatakana,
    /// The Kana Mode (Kana Lock) key.
    ///
    /// This key is used to enter hiragana mode (typically from romaji mode).
    KanaMode,
    /// The Kanji (Japanese name for ideographic characters of Chinese origin) Mode key.
    ///
    /// This key is typically used to switch to a hiragana keyboard for the purpose of converting input into kanji. (<code>KEYCODE_KANA</code>)
    KanjiMode,
    /// The Katakana (Japanese Kana characters) key.
    Katakana,
    /// The Roman characters function key.
    Romaji,
    /// The Zenkaku (Full-Width) Characters key.
    Zenkaku,
    /// The Zenkaku/Hankaku (full-width/half-width) toggle key. (<code>KEYCODE_ZENKAKU_HANKAKU</code>)
    ZenkakuHankaku,
    /// General purpose virtual function key, as index 1.
    Soft1,
    /// General purpose virtual function key, as index 2.
    Soft2,
    /// General purpose virtual function key, as index 3.
    Soft3,
    /// General purpose virtual function key, as index 4.
    Soft4,
    /// Select next (numerically or logically) lower channel. (<code>APPCOMMAND_MEDIA_CHANNEL_DOWN</code>, <code>KEYCODE_CHANNEL_DOWN</code>)
    ChannelDown,
    /// Select next (numerically or logically) higher channel. (<code>APPCOMMAND_MEDIA_CHANNEL_UP</code>, <code>KEYCODE_CHANNEL_UP</code>)
    ChannelUp,
    /// Close the current document or message (Note: This doesn’t close the application). (<code>APPCOMMAND_CLOSE</code>)
    Close,
    /// Open an editor to forward the current message. (<code>APPCOMMAND_FORWARD_MAIL</code>)
    MailForward,
    /// Open an editor to reply to the current message. (<code>APPCOMMAND_REPLY_TO_MAIL</code>)
    MailReply,
    /// Send the current message. (<code>APPCOMMAND_SEND_MAIL</code>)
    MailSend,
    /// Close the current media, for example to close a CD or DVD tray. (<code>KEYCODE_MEDIA_CLOSE</code>)
    MediaClose,
    /// Initiate or continue forward playback at faster than normal speed, or increase speed if already fast forwarding. (<code>APPCOMMAND_MEDIA_FAST_FORWARD</code>, <code>KEYCODE_MEDIA_FAST_FORWARD</code>)
    MediaFastForward,
    /// Pause the currently playing media. (<code>APPCOMMAND_MEDIA_PAUSE</code>, <code>KEYCODE_MEDIA_PAUSE</code>)
    ///
    /// Media controller devices should use this value rather than [`Pause`][NamedKey::Pause] for their pause keys.
    MediaPause,
    /// Initiate or continue media playback at normal speed, if not currently playing at normal speed. (<code>APPCOMMAND_MEDIA_PLAY</code>, <code>KEYCODE_MEDIA_PLAY</code>)
    MediaPlay,
    /// Toggle media between play and pause states. (<code>APPCOMMAND_MEDIA_PLAY_PAUSE</code>, <code>KEYCODE_MEDIA_PLAY_PAUSE</code>)
    MediaPlayPause,
    /// Initiate or resume recording of currently selected media. (<code>APPCOMMAND_MEDIA_RECORD</code>, <code>KEYCODE_MEDIA_RECORD</code>)
    MediaRecord,
    /// Initiate or continue reverse playback at faster than normal speed, or increase speed if already rewinding. (<code>APPCOMMAND_MEDIA_REWIND</code>, <code>KEYCODE_MEDIA_REWIND</code>)
    MediaRewind,
    /// Stop media playing, pausing, forwarding, rewinding, or recording, if not already stopped. (<code>APPCOMMAND_MEDIA_STOP</code>, <code>KEYCODE_MEDIA_STOP</code>)
    MediaStop,
    /// Seek to next media or program track. (<code>APPCOMMAND_MEDIA_NEXTTRACK</code>, <code>KEYCODE_MEDIA_NEXT</code>)
    MediaTrackNext,
    /// Seek to previous media or program track. (<code>APPCOMMAND_MEDIA_PREVIOUSTRACK</code>, <code>KEYCODE_MEDIA_PREVIOUS</code>)
    MediaTrackPrevious,
    /// Open a new document or message. (<code>APPCOMMAND_NEW</code>)
    New,
    /// Open an existing document or message. (<code>APPCOMMAND_OPEN</code>)
    Open,
    /// Print the current document or message. (<code>APPCOMMAND_PRINT</code>)
    Print,
    /// Save the current document or message. (<code>APPCOMMAND_SAVE</code>)
    Save,
    /// Spellcheck the current document or selection. (<code>APPCOMMAND_SPELL_CHECK</code>)
    SpellCheck,
    /// The <kbd>11</kbd> key found on media numpads that have buttons from <kbd>1</kbd> ... <kbd>12</kbd>.
    Key11,
    /// The <kbd>12</kbd> key found on media numpads that have buttons from <kbd>1</kbd> ... <kbd>12</kbd>.
    Key12,
    /// Adjust audio balance leftward. (<code>VK_AUDIO_BALANCE_LEFT</code>)
    AudioBalanceLeft,
    /// Adjust audio balance rightward. (<code>VK_AUDIO_BALANCE_RIGHT</code>)
    AudioBalanceRight,
    /// Decrease audio bass boost or cycle down through bass boost states. (<code>APPCOMMAND_BASS_DOWN</code>, <code>VK_BASS_BOOST_DOWN</code>)
    AudioBassBoostDown,
    /// Toggle bass boost on/off. (<code>APPCOMMAND_BASS_BOOST</code>)
    AudioBassBoostToggle,
    /// Increase audio bass boost or cycle up through bass boost states. (<code>APPCOMMAND_BASS_UP</code>, <code>VK_BASS_BOOST_UP</code>)
    AudioBassBoostUp,
    /// Adjust audio fader towards front. (<code>VK_FADER_FRONT</code>)
    AudioFaderFront,
    /// Adjust audio fader towards rear. (<code>VK_FADER_REAR</code>)
    AudioFaderRear,
    /// Advance surround audio mode to next available mode. (<code>VK_SURROUND_MODE_NEXT</code>)
    AudioSurroundModeNext,
    /// Decrease treble. (<code>APPCOMMAND_TREBLE_DOWN</code>)
    AudioTrebleDown,
    /// Increase treble. (<code>APPCOMMAND_TREBLE_UP</code>)
    AudioTrebleUp,
    /// Decrease audio volume. (<code>APPCOMMAND_VOLUME_DOWN</code>, <code>KEYCODE_VOLUME_DOWN</code>)
    AudioVolumeDown,
    /// Increase audio volume. (<code>APPCOMMAND_VOLUME_UP</code>, <code>KEYCODE_VOLUME_UP</code>)
    AudioVolumeUp,
    /// Toggle between muted state and prior volume level. (<code>APPCOMMAND_VOLUME_MUTE</code>, <code>KEYCODE_VOLUME_MUTE</code>)
    AudioVolumeMute,
    /// Toggle the microphone on/off. (<code>APPCOMMAND_MIC_ON_OFF_TOGGLE</code>)
    MicrophoneToggle,
    /// Decrease microphone volume. (<code>APPCOMMAND_MICROPHONE_VOLUME_DOWN</code>)
    MicrophoneVolumeDown,
    /// Increase microphone volume. (<code>APPCOMMAND_MICROPHONE_VOLUME_UP</code>)
    MicrophoneVolumeUp,
    /// Mute the microphone. (<code>APPCOMMAND_MICROPHONE_VOLUME_MUTE</code>, <code>KEYCODE_MUTE</code>)
    MicrophoneVolumeMute,
    /// Show correction list when a word is incorrectly identified. (<code>APPCOMMAND_CORRECTION_LIST</code>)
    SpeechCorrectionList,
    /// Toggle between dictation mode and command/control mode. (<code>APPCOMMAND_DICTATE_OR_COMMAND_CONTROL_TOGGLE</code>)
    SpeechInputToggle,
    /// The first generic "LaunchApplication" key.
    ///
    /// This is commonly associated with launching "My Computer", and may have a computer symbol on the key. (<code>APPCOMMAND_LAUNCH_APP1</code>)
    LaunchApplication1,
    /// The second generic "LaunchApplication" key.
    ///
    /// This is commonly associated with launching "Calculator", and may have a calculator symbol on the key. (<code>APPCOMMAND_LAUNCH_APP2</code>, <code>KEYCODE_CALCULATOR</code>)
    LaunchApplication2,
    /// The "Calendar" key. (<code>KEYCODE_CALENDAR</code>)
    LaunchCalendar,
    /// The "Contacts" key. (<code>KEYCODE_CONTACTS</code>)
    LaunchContacts,
    /// The "Mail" key. (<code>APPCOMMAND_LAUNCH_MAIL</code>)
    LaunchMail,
    /// The "Media Player" key. (<code>APPCOMMAND_LAUNCH_MEDIA_SELECT</code>)
    LaunchMediaPlayer,
    /// The "Music Player" key.
    LaunchMusicPlayer,
    /// The "Phone" key.
    LaunchPhone,
    /// The "Screen Saver" key.
    LaunchScreenSaver,
    /// The "Spreadsheet" key.
    LaunchSpreadsheet,
    /// The "Web Browser" key.
    LaunchWebBrowser,
    /// The "WebCam" key.
    LaunchWebCam,
    /// The "Word Processor" key.
    LaunchWordProcessor,
    /// Navigate to previous content or page in current history. (<code>APPCOMMAND_BROWSER_BACKWARD</code>)
    BrowserBack,
    /// Open the list of browser favorites. (<code>APPCOMMAND_BROWSER_FAVORITES</code>)
    BrowserFavorites,
    /// Navigate to next content or page in current history. (<code>APPCOMMAND_BROWSER_FORWARD</code>)
    BrowserForward,
    /// Go to the user’s preferred home page. (<code>APPCOMMAND_BROWSER_HOME</code>)
    BrowserHome,
    /// Refresh the current page or content. (<code>APPCOMMAND_BROWSER_REFRESH</code>)
    BrowserRefresh,
    /// Call up the user’s preferred search page. (<code>APPCOMMAND_BROWSER_SEARCH</code>)
    BrowserSearch,
    /// Stop loading the current page or content. (<code>APPCOMMAND_BROWSER_STOP</code>)
    BrowserStop,
    /// The Application switch key, which provides a list of recent apps to switch between. (<code>KEYCODE_APP_SWITCH</code>)
    AppSwitch,
    /// The Call key. (<code>KEYCODE_CALL</code>)
    Call,
    /// The Camera key. (<code>KEYCODE_CAMERA</code>)
    Camera,
    /// The Camera focus key. (<code>KEYCODE_FOCUS</code>)
    CameraFocus,
    /// The End Call key. (<code>KEYCODE_ENDCALL</code>)
    EndCall,
    /// The Back key. (<code>KEYCODE_BACK</code>)
    GoBack,
    /// The Home key, which goes to the phone’s main screen. (<code>KEYCODE_HOME</code>)
    GoHome,
    /// The Headset Hook key. (<code>KEYCODE_HEADSETHOOK</code>)
    HeadsetHook,
    /// The Last Number Redial key.
    LastNumberRedial,
    /// The Notification key. (<code>KEYCODE_NOTIFICATION</code>)
    Notification,
    /// Toggle between manner mode state: silent, vibrate, ring, ... (<code>KEYCODE_MANNER_MODE</code>)
    MannerMode,
    /// The Voice Dial key.
    VoiceDial,
    /// Switch to viewing TV. (<code>KEYCODE_TV</code>)
    TV,
    /// TV 3D Mode. (<code>KEYCODE_3D_MODE</code>)
    TV3DMode,
    /// Toggle between antenna and cable input. (<code>KEYCODE_TV_ANTENNA_CABLE</code>)
    TVAntennaCable,
    /// Audio description. (<code>KEYCODE_TV_AUDIO_DESCRIPTION</code>)
    TVAudioDescription,
    /// Audio description mixing volume down. (<code>KEYCODE_TV_AUDIO_DESCRIPTION_MIX_DOWN</code>)
    TVAudioDescriptionMixDown,
    /// Audio description mixing volume up. (<code>KEYCODE_TV_AUDIO_DESCRIPTION_MIX_UP</code>)
    TVAudioDescriptionMixUp,
    /// Contents menu. (<code>KEYCODE_TV_CONTENTS_MENU</code>)
    TVContentsMenu,
    /// Contents menu. (<code>KEYCODE_TV_DATA_SERVICE</code>)
    TVDataService,
    /// Switch the input mode on an external TV. (<code>KEYCODE_TV_INPUT</code>)
    TVInput,
    /// Switch to component input #1. (<code>KEYCODE_TV_INPUT_COMPONENT_1</code>)
    TVInputComponent1,
    /// Switch to component input #2. (<code>KEYCODE_TV_INPUT_COMPONENT_2</code>)
    TVInputComponent2,
    /// Switch to composite input #1. (<code>KEYCODE_TV_INPUT_COMPOSITE_1</code>)
    TVInputComposite1,
    /// Switch to composite input #2. (<code>KEYCODE_TV_INPUT_COMPOSITE_2</code>)
    TVInputComposite2,
    /// Switch to HDMI input #1. (<code>KEYCODE_TV_INPUT_HDMI_1</code>)
    TVInputHDMI1,
    /// Switch to HDMI input #2. (<code>KEYCODE_TV_INPUT_HDMI_2</code>)
    TVInputHDMI2,
    /// Switch to HDMI input #3. (<code>KEYCODE_TV_INPUT_HDMI_3</code>)
    TVInputHDMI3,
    /// Switch to HDMI input #4. (<code>KEYCODE_TV_INPUT_HDMI_4</code>)
    TVInputHDMI4,
    /// Switch to VGA input #1. (<code>KEYCODE_TV_INPUT_VGA_1</code>)
    TVInputVGA1,
    /// Media context menu. (<code>KEYCODE_TV_MEDIA_CONTEXT_MENU</code>)
    TVMediaContext,
    /// Toggle network. (<code>KEYCODE_TV_NETWORK</code>)
    TVNetwork,
    /// Number entry. (<code>KEYCODE_TV_NUMBER_ENTRY</code>)
    TVNumberEntry,
    /// Toggle the power on an external TV. (<code>KEYCODE_TV_POWER</code>)
    TVPower,
    /// Radio. (<code>KEYCODE_TV_RADIO_SERVICE</code>)
    TVRadioService,
    /// Satellite. (<code>KEYCODE_TV_SATELLITE</code>)
    TVSatellite,
    /// Broadcast Satellite. (<code>KEYCODE_TV_SATELLITE_BS</code>)
    TVSatelliteBS,
    /// Communication Satellite. (<code>KEYCODE_TV_SATELLITE_CS</code>)
    TVSatelliteCS,
    /// Toggle between available satellites. (<code>KEYCODE_TV_SATELLITE_SERVICE</code>)
    TVSatelliteToggle,
    /// Analog Terrestrial. (<code>KEYCODE_TV_TERRESTRIAL_ANALOG</code>)
    TVTerrestrialAnalog,
    /// Digital Terrestrial. (<code>KEYCODE_TV_TERRESTRIAL_DIGITAL</code>)
    TVTerrestrialDigital,
    /// Timer programming. (<code>KEYCODE_TV_TIMER_PROGRAMMING</code>)
    TVTimer,
    /// Switch the input mode on an external AVR (audio/video receiver). (<code>KEYCODE_AVR_INPUT</code>)
    AVRInput,
    /// Toggle the power on an external AVR (audio/video receiver). (<code>KEYCODE_AVR_POWER</code>)
    AVRPower,
    /// General purpose color-coded media function key, as index 0 (red). (<code>VK_COLORED_KEY_0</code>, <code>KEYCODE_PROG_RED</code>)
    ColorF0Red,
    /// General purpose color-coded media function key, as index 1 (green). (<code>VK_COLORED_KEY_1</code>, <code>KEYCODE_PROG_GREEN</code>)
    ColorF1Green,
    /// General purpose color-coded media function key, as index 2 (yellow). (<code>VK_COLORED_KEY_2</code>, <code>KEYCODE_PROG_YELLOW</code>)
    ColorF2Yellow,
    /// General purpose color-coded media function key, as index 3 (blue). (<code>VK_COLORED_KEY_3</code>, <code>KEYCODE_PROG_BLUE</code>)
    ColorF3Blue,
    /// General purpose color-coded media function key, as index 4 (grey). (<code>VK_COLORED_KEY_4</code>)
    ColorF4Grey,
    /// General purpose color-coded media function key, as index 5 (brown). (<code>VK_COLORED_KEY_5</code>)
    ColorF5Brown,
    /// Toggle the display of Closed Captions. (<code>VK_CC</code>, <code>KEYCODE_CAPTIONS</code>)
    ClosedCaptionToggle,
    /// Adjust brightness of device, by toggling between or cycling through states. (<code>VK_DIMMER</code>)
    Dimmer,
    /// Swap video sources. (<code>VK_DISPLAY_SWAP</code>)
    DisplaySwap,
    /// Select Digital Video Rrecorder. (<code>KEYCODE_DVR</code>)
    DVR,
    /// Exit the current application. (<code>VK_EXIT</code>)
    Exit,
    /// Clear program or content stored as favorite 0. (<code>VK_CLEAR_FAVORITE_0</code>)
    FavoriteClear0,
    /// Clear program or content stored as favorite 1. (<code>VK_CLEAR_FAVORITE_1</code>)
    FavoriteClear1,
    /// Clear program or content stored as favorite 2. (<code>VK_CLEAR_FAVORITE_2</code>)
    FavoriteClear2,
    /// Clear program or content stored as favorite 3. (<code>VK_CLEAR_FAVORITE_3</code>)
    FavoriteClear3,
    /// Select (recall) program or content stored as favorite 0. (<code>VK_RECALL_FAVORITE_0</code>)
    FavoriteRecall0,
    /// Select (recall) program or content stored as favorite 1. (<code>VK_RECALL_FAVORITE_1</code>)
    FavoriteRecall1,
    /// Select (recall) program or content stored as favorite 2. (<code>VK_RECALL_FAVORITE_2</code>)
    FavoriteRecall2,
    /// Select (recall) program or content stored as favorite 3. (<code>VK_RECALL_FAVORITE_3</code>)
    FavoriteRecall3,
    /// Store current program or content as favorite 0. (<code>VK_STORE_FAVORITE_0</code>)
    FavoriteStore0,
    /// Store current program or content as favorite 1. (<code>VK_STORE_FAVORITE_1</code>)
    FavoriteStore1,
    /// Store current program or content as favorite 2. (<code>VK_STORE_FAVORITE_2</code>)
    FavoriteStore2,
    /// Store current program or content as favorite 3. (<code>VK_STORE_FAVORITE_3</code>)
    FavoriteStore3,
    /// Toggle display of program or content guide. (<code>VK_GUIDE</code>, <code>KEYCODE_GUIDE</code>)
    Guide,
    /// If guide is active and displayed, then display next day’s content. (<code>VK_NEXT_DAY</code>)
    GuideNextDay,
    /// If guide is active and displayed, then display previous day’s content. (<code>VK_PREV_DAY</code>)
    GuidePreviousDay,
    /// Toggle display of information about currently selected context or media. (<code>VK_INFO</code>, <code>KEYCODE_INFO</code>)
    Info,
    /// Toggle instant replay. (<code>VK_INSTANT_REPLAY</code>)
    InstantReplay,
    /// Launch linked content, if available and appropriate. (<code>VK_LINK</code>)
    Link,
    /// List the current program. (<code>VK_LIST</code>)
    ListProgram,
    /// Toggle display listing of currently available live content or programs. (<code>VK_LIVE</code>)
    LiveContent,
    /// Lock or unlock current content or program. (<code>VK_LOCK</code>)
    Lock,
    /// Show a list of media applications: audio/video players and image viewers. (<code>VK_APPS</code>)
    ///
    /// Do not confuse this key value with the Windows' <code>VK_APPS</code> / <code>VK_CONTEXT_MENU</code> key, which is encoded as [`ContextMenu`][NamedKey::ContextMenu].
    MediaApps,
    /// Audio track key. (<code>KEYCODE_MEDIA_AUDIO_TRACK</code>)
    MediaAudioTrack,
    /// Select previously selected channel or media. (<code>VK_LAST</code>, <code>KEYCODE_LAST_CHANNEL</code>)
    MediaLast,
    /// Skip backward to next content or program. (<code>KEYCODE_MEDIA_SKIP_BACKWARD</code>)
    MediaSkipBackward,
    /// Skip forward to next content or program. (<code>VK_SKIP</code>, <code>KEYCODE_MEDIA_SKIP_FORWARD</code>)
    MediaSkipForward,
    /// Step backward to next content or program. (<code>KEYCODE_MEDIA_STEP_BACKWARD</code>)
    MediaStepBackward,
    /// Step forward to next content or program. (<code>KEYCODE_MEDIA_STEP_FORWARD</code>)
    MediaStepForward,
    /// Media top menu. (<code>KEYCODE_MEDIA_TOP_MENU</code>)
    MediaTopMenu,
    /// Navigate in. (<code>KEYCODE_NAVIGATE_IN</code>)
    NavigateIn,
    /// Navigate to next key. (<code>KEYCODE_NAVIGATE_NEXT</code>)
    NavigateNext,
    /// Navigate out. (<code>KEYCODE_NAVIGATE_OUT</code>)
    NavigateOut,
    /// Navigate to previous key. (<code>KEYCODE_NAVIGATE_PREVIOUS</code>)
    NavigatePrevious,
    /// Cycle to next favorite channel (in favorites list). (<code>VK_NEXT_FAVORITE_CHANNEL</code>)
    NextFavoriteChannel,
    /// Cycle to next user profile (if there are multiple user profiles). (<code>VK_USER</code>)
    NextUserProfile,
    /// Access on-demand content or programs. (<code>VK_ON_DEMAND</code>)
    OnDemand,
    /// Pairing key to pair devices. (<code>KEYCODE_PAIRING</code>)
    Pairing,
    /// Move picture-in-picture window down. (<code>VK_PINP_DOWN</code>)
    PinPDown,
    /// Move picture-in-picture window. (<code>VK_PINP_MOVE</code>)
    PinPMove,
    /// Toggle display of picture-in-picture window. (<code>VK_PINP_TOGGLE</code>)
    PinPToggle,
    /// Move picture-in-picture window up. (<code>VK_PINP_UP</code>)
    PinPUp,
    /// Decrease media playback speed. (<code>VK_PLAY_SPEED_DOWN</code>)
    PlaySpeedDown,
    /// Reset playback to normal speed. (<code>VK_PLAY_SPEED_RESET</code>)
    PlaySpeedReset,
    /// Increase media playback speed. (<code>VK_PLAY_SPEED_UP</code>)
    PlaySpeedUp,
    /// Toggle random media or content shuffle mode. (<code>VK_RANDOM_TOGGLE</code>)
    RandomToggle,
    /// Not a physical key, but this key code is sent when the remote control battery is low. (<code>VK_RC_LOW_BATTERY</code>)
    RcLowBattery,
    /// Toggle or cycle between media recording speeds. (<code>VK_RECORD_SPEED_NEXT</code>)
    RecordSpeedNext,
    /// Toggle RF (radio frequency) input bypass mode (pass RF input directly to the RF output). (<code>VK_RF_BYPASS</code>)
    RfBypass,
    /// Toggle scan channels mode. (<code>VK_SCAN_CHANNELS_TOGGLE</code>)
    ScanChannelsToggle,
    /// Advance display screen mode to next available mode. (<code>VK_SCREEN_MODE_NEXT</code>)
    ScreenModeNext,
    /// Toggle display of device settings screen. (<code>VK_SETTINGS</code>, <code>KEYCODE_SETTINGS</code>)
    Settings,
    /// Toggle split screen mode. (<code>VK_SPLIT_SCREEN_TOGGLE</code>)
    SplitScreenToggle,
    /// Switch the input mode on an external STB (set top box). (<code>KEYCODE_STB_INPUT</code>)
    STBInput,
    /// Toggle the power on an external STB (set top box). (<code>KEYCODE_STB_POWER</code>)
    STBPower,
    /// Toggle display of subtitles, if available. (<code>VK_SUBTITLE</code>)
    Subtitle,
    /// Toggle display of teletext, if available (<code>VK_TELETEXT</code>, <code>KEYCODE_TV_TELETEXT</code>).
    Teletext,
    /// Advance video mode to next available mode. (<code>VK_VIDEO_MODE_NEXT</code>)
    VideoModeNext,
    /// Cause device to identify itself in some manner, e.g., audibly or visibly. (<code>VK_WINK</code>)
    Wink,
    /// Toggle between full-screen and scaled content, or alter magnification level. (<code>VK_ZOOM</code>, <code>KEYCODE_TV_ZOOM_MODE</code>)
    ZoomToggle,
    /// The F1 key, a general purpose function key, as index 1.
    F1,
    /// The F2 key, a general purpose function key, as index 2.
    F2,
    /// The F3 key, a general purpose function key, as index 3.
    F3,
    /// The F4 key, a general purpose function key, as index 4.
    F4,
    /// The F5 key, a general purpose function key, as index 5.
    F5,
    /// The F6 key, a general purpose function key, as index 6.
    F6,
    /// The F7 key, a general purpose function key, as index 7.
    F7,
    /// The F8 key, a general purpose function key, as index 8.
    F8,
    /// The F9 key, a general purpose function key, as index 9.
    F9,
    /// The F10 key, a general purpose function key, as index 10.
    F10,
    /// The F11 key, a general purpose function key, as index 11.
    F11,
    /// The F12 key, a general purpose function key, as index 12.
    F12,
    /// The F13 key, a general purpose function key, as index 13.
    F13,
    /// The F14 key, a general purpose function key, as index 14.
    F14,
    /// The F15 key, a general purpose function key, as index 15.
    F15,
    /// The F16 key, a general purpose function key, as index 16.
    F16,
    /// The F17 key, a general purpose function key, as index 17.
    F17,
    /// The F18 key, a general purpose function key, as index 18.
    F18,
    /// The F19 key, a general purpose function key, as index 19.
    F19,
    /// The F20 key, a general purpose function key, as index 20.
    F20,
    /// The F21 key, a general purpose function key, as index 21.
    F21,
    /// The F22 key, a general purpose function key, as index 22.
    F22,
    /// The F23 key, a general purpose function key, as index 23.
    F23,
    /// The F24 key, a general purpose function key, as index 24.
    F24,
    /// The F25 key, a general purpose function key, as index 25.
    F25,
    /// The F26 key, a general purpose function key, as index 26.
    F26,
    /// The F27 key, a general purpose function key, as index 27.
    F27,
    /// The F28 key, a general purpose function key, as index 28.
    F28,
    /// The F29 key, a general purpose function key, as index 29.
    F29,
    /// The F30 key, a general purpose function key, as index 30.
    F30,
    /// The F31 key, a general purpose function key, as index 31.
    F31,
    /// The F32 key, a general purpose function key, as index 32.
    F32,
    /// The F33 key, a general purpose function key, as index 33.
    F33,
    /// The F34 key, a general purpose function key, as index 34.
    F34,
    /// The F35 key, a general purpose function key, as index 35.
    F35,
}

#[derive(Debug, Clone)]
pub enum Key {
    Named(NamedKey),
    Character(SmolStr),
    Dead(Option<char>),
    Unknown(Option<u32>),
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Self::Named(l), &Self::Named(r)) => l == r,
            (Self::Character(l), Self::Character(r)) => l == r,
            (&Self::Dead(Some(l)), &Self::Dead(Some(r))) => l == r,
            (&Self::Unknown(Some(l)), &Self::Unknown(Some(r))) => l == r,
            _ => false,
        }
    }
}

impl Key {
    pub fn eq_ignore_ascii_case(&self, other: &Self) -> bool {
        match (self, other) {
            (&Self::Named(l), &Self::Named(r)) => l == r,
            (Self::Character(l), Self::Character(r)) => l.eq_ignore_ascii_case(r),
            (&Self::Dead(Some(l)), &Self::Dead(Some(r))) => l.eq_ignore_ascii_case(&r),
            (&Self::Unknown(Some(l)), &Self::Unknown(Some(r))) => l == r,
            _ => false,
        }
    }
}

impl From<NamedKey> for Key {
    #[inline]
    fn from(key: NamedKey) -> Self {
        Self::Named(key)
    }
}

impl From<SmolStr> for Key {
    #[inline]
    fn from(c: SmolStr) -> Self {
        Self::Character(c)
    }
}

impl From<&str> for Key {
    #[inline]
    fn from(c: &str) -> Self {
        Self::Character(c.into())
    }
}

bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct KeyLocations: u8 {
        const STANDARD = 0x01;
        const LEFT = 0x02;
        const RIGHT = 0x04;
        const NUMPAD = 0x08;
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyLocation {
    #[default]
    Standard,
    Left,
    Right,
    Numpad,
}

impl From<KeyLocation> for KeyLocations {
    #[inline]
    fn from(location: KeyLocation) -> Self {
        match location {
            KeyLocation::Standard => KeyLocations::STANDARD,
            KeyLocation::Left => KeyLocations::LEFT,
            KeyLocation::Right => KeyLocations::RIGHT,
            KeyLocation::Numpad => KeyLocations::NUMPAD,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Shortcut {
    pub modifiers: Modifiers,
    pub key: Key,
    pub location: Option<KeyLocation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyEventMatch {
    True,
    ConsumeOnly,
    False,
}

impl KeyEvent {
    pub fn matches(&self, shortcut: &Shortcut) -> KeyEventMatch {
        if let KeyEvent::Pressed {
            key,
            location,
            modifiers,
            repeat: false,
            ..
        } = self
        {
            let matches = match shortcut.location {
                Some(shortcut_location) => {
                    key.eq_ignore_ascii_case(&shortcut.key)
                        && (*location == shortcut_location)
                        && modifiers.matches(shortcut.modifiers)
                }
                None => {
                    key.eq_ignore_ascii_case(&shortcut.key) && modifiers.matches(shortcut.modifiers)
                }
            };

            if matches {
                return KeyEventMatch::True;
            }
        } else if let KeyEvent::Pressed {
            key,
            location,
            modifiers,
            repeat: true,
            ..
        }
        | KeyEvent::Released {
            key,
            location,
            modifiers,
            ..
        } = self
        {
            let matches = match shortcut.location {
                Some(shortcut_location) => {
                    key.eq_ignore_ascii_case(&shortcut.key)
                        && (*location == shortcut_location)
                        && modifiers.matches(shortcut.modifiers)
                }
                None => {
                    key.eq_ignore_ascii_case(&shortcut.key) && modifiers.matches(shortcut.modifiers)
                }
            };

            if matches {
                return KeyEventMatch::ConsumeOnly;
            }
        }

        KeyEventMatch::False
    }
}

bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MouseButtons: u8 {
        const PRIMARY = 0x01;
        const SECONDARY = 0x02;
        const MIDDLE = 0x04;
        const BACK = 0x08;
        const FORWARD = 0x10;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Primary,
    Secondary,
    Middle,
    Back,
    Forward,
}

impl From<MouseButton> for MouseButtons {
    #[inline]
    fn from(button: MouseButton) -> Self {
        match button {
            MouseButton::Primary => MouseButtons::PRIMARY,
            MouseButton::Secondary => MouseButtons::SECONDARY,
            MouseButton::Middle => MouseButtons::MIDDLE,
            MouseButton::Back => MouseButtons::BACK,
            MouseButton::Forward => MouseButtons::FORWARD,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ScrollDelta {
    Pixel(Vec2<Pixel>),
    Point(Vec2<Point>),
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    KeyPressed {
        key: Key,
        location: KeyLocation,
        text: Option<SmolStr>,
        repeat: bool,
    },
    KeyReleased {
        key: Key,
        location: KeyLocation,
        text: Option<SmolStr>,
    },
    CursorMoved {
        position: Vec2<Pixel>,
    },
    ButtonPressed {
        button: MouseButton,
    },
    ButtonReleased {
        button: MouseButton,
    },
    Scrolled {
        delta: ScrollDelta,
    },
}

#[derive(Debug, Clone)]
pub enum KeyEvent {
    Pressed {
        key: Key,
        location: KeyLocation,
        text: Option<SmolStr>,
        modifiers: Modifiers,
        repeat: bool,
    },
    Released {
        key: Key,
        location: KeyLocation,
        text: Option<SmolStr>,
        modifiers: Modifiers,
    },
}

#[derive(Debug, Default, Clone)]
pub struct InputState {
    modifiers: ModifiersState,
    pressed_keys: Vec<(Key, KeyLocation)>,
    key_events: Vec<KeyEvent>,

    prev_position: Option<Vec2<Pixel>>,
    position: Vec2<Pixel>,

    prev_pressed_buttons: MouseButtons,
    pressed_buttons: MouseButtons,

    scroll_delta: Vec2<Pixel>,
}

impl InputState {
    #[inline]
    pub(crate) fn on_event(&mut self, event: InputEvent, scale_factor: f32) {
        match event {
            InputEvent::KeyPressed {
                key,
                location,
                text,
                repeat,
            } => {
                match key {
                    Key::Named(NamedKey::Control) => self.modifiers.insert_control(location),
                    Key::Named(NamedKey::Shift) => self.modifiers.insert_shift(location),
                    Key::Named(NamedKey::Alt) => self.modifiers.insert_alt(location),
                    Key::Named(NamedKey::AltGraph) => self.modifiers.insert_alt_graph(location),
                    Key::Named(NamedKey::Meta) => self.modifiers.insert_meta(location),
                    _ => (),
                }

                let tuple = (key.clone(), location);
                if !self.pressed_keys.contains(&tuple) {
                    self.pressed_keys.push(tuple);
                }

                self.key_events.push(KeyEvent::Pressed {
                    key,
                    location,
                    text,
                    modifiers: self.modifiers(),
                    repeat,
                });
            }
            InputEvent::KeyReleased {
                key,
                location,
                text,
            } => {
                match key {
                    Key::Named(NamedKey::Control) => self.modifiers.remove_control(location),
                    Key::Named(NamedKey::Shift) => self.modifiers.remove_shift(location),
                    Key::Named(NamedKey::Alt) => self.modifiers.remove_alt(location),
                    Key::Named(NamedKey::AltGraph) => self.modifiers.remove_alt_graph(location),
                    Key::Named(NamedKey::Meta) => self.modifiers.remove_meta(location),
                    _ => (),
                }

                self.pressed_keys
                    .retain(|&(ref k, l)| *k != key || l != location);

                self.key_events.push(KeyEvent::Released {
                    key,
                    location,
                    text,
                    modifiers: self.modifiers(),
                });
            }
            InputEvent::CursorMoved { position } => self.position = position,
            InputEvent::ButtonPressed { button } => self.pressed_buttons.insert(button.into()),
            InputEvent::ButtonReleased { button } => self.pressed_buttons.remove(button.into()),
            InputEvent::Scrolled { delta } => match delta {
                ScrollDelta::Pixel(delta) => self.scroll_delta += delta,
                ScrollDelta::Point(delta) => self.scroll_delta += delta.to_pixel(scale_factor),
            },
        }
    }

    #[inline]
    pub(crate) fn end_frame(&mut self) {
        self.key_events.clear();
        self.prev_position = Some(self.position);
        self.prev_pressed_buttons = self.pressed_buttons;
        self.scroll_delta = Vec2::ZERO;
    }

    #[must_use]
    #[inline]
    pub fn modifiers(&self) -> Modifiers {
        self.modifiers.into()
    }

    #[must_use]
    pub fn key_pressed(&self, key: &Key) -> bool {
        self.pressed_keys.iter().find(|(k, _)| k == key).is_some()
    }

    #[must_use]
    pub fn key_location(&self, key: &Key) -> Option<KeyLocation> {
        self.pressed_keys
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, l)| *l)
    }

    #[must_use]
    #[inline]
    pub fn key_events(&self) -> &[KeyEvent] {
        &self.key_events
    }

    #[inline]
    pub fn retain_key_events(&mut self, f: impl FnMut(&KeyEvent) -> bool) {
        self.key_events.retain(f);
    }

    pub fn consume_shortcut(&mut self, shortcut: &Shortcut) -> bool {
        let mut shortcut_pressed = false;
        self.retain_key_events(|event| match event.matches(shortcut) {
            KeyEventMatch::True => {
                shortcut_pressed = true;
                false
            }
            KeyEventMatch::ConsumeOnly => false,
            KeyEventMatch::False => true,
        });
        shortcut_pressed
    }

    #[must_use]
    #[inline]
    pub fn cursor_position(&self) -> Vec2<Pixel> {
        self.position
    }

    #[must_use]
    #[inline]
    pub fn cursor_delta(&self) -> Vec2<Pixel> {
        if let Some(prev_position) = self.prev_position {
            self.position - prev_position
        } else {
            Vec2::ZERO
        }
    }

    #[must_use]
    #[inline]
    pub fn pressed_buttons(&self) -> MouseButtons {
        self.pressed_buttons
    }

    #[must_use]
    #[inline]
    pub fn clicked_buttons(&self) -> MouseButtons {
        self.pressed_buttons & !self.prev_pressed_buttons
    }

    #[must_use]
    #[inline]
    pub fn released_buttons(&self) -> MouseButtons {
        self.prev_pressed_buttons & !self.pressed_buttons
    }

    #[must_use]
    #[inline]
    pub fn scroll_delta(&self) -> Vec2<Pixel> {
        self.scroll_delta
    }
}
