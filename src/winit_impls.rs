use crate::{IntoFloat, Pixel, Vec2, input};

impl From<winit::keyboard::ModifiersState> for input::Modifiers {
    fn from(state: winit::keyboard::ModifiersState) -> Self {
        let mut modifiers = input::Modifiers::empty();
        if state.contains(winit::keyboard::ModifiersState::CONTROL) {
            modifiers |= input::Modifiers::CONTROL;
        }
        if state.contains(winit::keyboard::ModifiersState::SHIFT) {
            modifiers |= input::Modifiers::SHIFT;
        }
        if state.contains(winit::keyboard::ModifiersState::ALT) {
            modifiers |= input::Modifiers::ALT;
        }
        if state.contains(winit::keyboard::ModifiersState::SUPER) {
            modifiers |= input::Modifiers::META;
        }
        modifiers
    }
}

impl From<winit::event::Modifiers> for input::Modifiers {
    #[inline]
    fn from(modifiers: winit::event::Modifiers) -> Self {
        modifiers.state().into()
    }
}

impl TryFrom<winit::event::MouseButton> for input::MouseButton {
    type Error = u16;

    fn try_from(button: winit::event::MouseButton) -> Result<Self, Self::Error> {
        match button {
            winit::event::MouseButton::Left => Ok(input::MouseButton::Primary),
            winit::event::MouseButton::Right => Ok(input::MouseButton::Secondary),
            winit::event::MouseButton::Middle => Ok(input::MouseButton::Middle),
            winit::event::MouseButton::Back => Ok(input::MouseButton::Back),
            winit::event::MouseButton::Forward => Ok(input::MouseButton::Forward),
            winit::event::MouseButton::Other(other) => Err(other),
        }
    }
}

impl From<winit::dpi::PhysicalPosition<f64>> for Vec2<Pixel> {
    #[inline]
    fn from(position: winit::dpi::PhysicalPosition<f64>) -> Self {
        Vec2 {
            x: position.x.px(),
            y: position.y.px(),
        }
    }
}

impl From<winit::keyboard::NamedKey> for input::Key {
    fn from(key: winit::keyboard::NamedKey) -> Self {
        use input::NamedKey as INK;
        use winit::keyboard::NamedKey as WNK;

        match key {
            WNK::Alt => INK::Alt.into(),
            WNK::AltGraph => INK::AltGraph.into(),
            WNK::CapsLock => INK::CapsLock.into(),
            WNK::Control => INK::Control.into(),
            WNK::Fn => INK::Fn.into(),
            WNK::FnLock => INK::FnLock.into(),
            WNK::NumLock => INK::NumLock.into(),
            WNK::ScrollLock => INK::ScrollLock.into(),
            WNK::Shift => INK::Shift.into(),
            WNK::Symbol => INK::Symbol.into(),
            WNK::SymbolLock => INK::SymbolLock.into(),
            WNK::Meta => INK::Meta.into(),
            WNK::Hyper => INK::Meta.into(),
            WNK::Super => INK::Meta.into(),
            WNK::Enter => INK::Enter.into(),
            WNK::Tab => INK::Tab.into(),
            WNK::Space => " ".into(),
            WNK::ArrowDown => INK::ArrowDown.into(),
            WNK::ArrowLeft => INK::ArrowLeft.into(),
            WNK::ArrowRight => INK::ArrowRight.into(),
            WNK::ArrowUp => INK::ArrowUp.into(),
            WNK::End => INK::End.into(),
            WNK::Home => INK::Home.into(),
            WNK::PageDown => INK::PageDown.into(),
            WNK::PageUp => INK::PageUp.into(),
            WNK::Backspace => INK::Backspace.into(),
            WNK::Clear => INK::Clear.into(),
            WNK::Copy => INK::Copy.into(),
            WNK::CrSel => INK::CrSel.into(),
            WNK::Cut => INK::Cut.into(),
            WNK::Delete => INK::Delete.into(),
            WNK::EraseEof => INK::EraseEof.into(),
            WNK::ExSel => INK::ExSel.into(),
            WNK::Insert => INK::Insert.into(),
            WNK::Paste => INK::Paste.into(),
            WNK::Redo => INK::Redo.into(),
            WNK::Undo => INK::Undo.into(),
            WNK::Accept => INK::Accept.into(),
            WNK::Again => INK::Again.into(),
            WNK::Attn => INK::Attn.into(),
            WNK::Cancel => INK::Cancel.into(),
            WNK::ContextMenu => INK::ContextMenu.into(),
            WNK::Escape => INK::Escape.into(),
            WNK::Execute => INK::Execute.into(),
            WNK::Find => INK::Find.into(),
            WNK::Help => INK::Help.into(),
            WNK::Pause => INK::Pause.into(),
            WNK::Play => INK::Play.into(),
            WNK::Props => INK::Props.into(),
            WNK::Select => INK::Select.into(),
            WNK::ZoomIn => INK::ZoomIn.into(),
            WNK::ZoomOut => INK::ZoomOut.into(),
            WNK::BrightnessDown => INK::BrightnessDown.into(),
            WNK::BrightnessUp => INK::BrightnessUp.into(),
            WNK::Eject => INK::Eject.into(),
            WNK::LogOff => INK::LogOff.into(),
            WNK::Power => INK::Power.into(),
            WNK::PowerOff => INK::PowerOff.into(),
            WNK::PrintScreen => INK::PrintScreen.into(),
            WNK::Hibernate => INK::Hibernate.into(),
            WNK::Standby => INK::Standby.into(),
            WNK::WakeUp => INK::WakeUp.into(),
            WNK::AllCandidates => INK::AllCandidates.into(),
            WNK::Alphanumeric => INK::Alphanumeric.into(),
            WNK::CodeInput => INK::CodeInput.into(),
            WNK::Compose => INK::Compose.into(),
            WNK::Convert => INK::Convert.into(),
            WNK::FinalMode => INK::FinalMode.into(),
            WNK::GroupFirst => INK::GroupFirst.into(),
            WNK::GroupLast => INK::GroupLast.into(),
            WNK::GroupNext => INK::GroupNext.into(),
            WNK::GroupPrevious => INK::GroupPrevious.into(),
            WNK::ModeChange => INK::ModeChange.into(),
            WNK::NextCandidate => INK::NextCandidate.into(),
            WNK::NonConvert => INK::NonConvert.into(),
            WNK::PreviousCandidate => INK::PreviousCandidate.into(),
            WNK::Process => INK::Process.into(),
            WNK::SingleCandidate => INK::SingleCandidate.into(),
            WNK::HangulMode => INK::HangulMode.into(),
            WNK::HanjaMode => INK::HanjaMode.into(),
            WNK::JunjaMode => INK::JunjaMode.into(),
            WNK::Eisu => INK::Eisu.into(),
            WNK::Hankaku => INK::Hankaku.into(),
            WNK::Hiragana => INK::Hiragana.into(),
            WNK::HiraganaKatakana => INK::HiraganaKatakana.into(),
            WNK::KanaMode => INK::KanaMode.into(),
            WNK::KanjiMode => INK::KanjiMode.into(),
            WNK::Katakana => INK::Katakana.into(),
            WNK::Romaji => INK::Romaji.into(),
            WNK::Zenkaku => INK::Zenkaku.into(),
            WNK::ZenkakuHankaku => INK::ZenkakuHankaku.into(),
            WNK::Soft1 => INK::Soft1.into(),
            WNK::Soft2 => INK::Soft2.into(),
            WNK::Soft3 => INK::Soft3.into(),
            WNK::Soft4 => INK::Soft4.into(),
            WNK::ChannelDown => INK::ChannelDown.into(),
            WNK::ChannelUp => INK::ChannelUp.into(),
            WNK::Close => INK::Close.into(),
            WNK::MailForward => INK::MailForward.into(),
            WNK::MailReply => INK::MailReply.into(),
            WNK::MailSend => INK::MailSend.into(),
            WNK::MediaClose => INK::MediaClose.into(),
            WNK::MediaFastForward => INK::MediaFastForward.into(),
            WNK::MediaPause => INK::MediaPause.into(),
            WNK::MediaPlay => INK::MediaPlay.into(),
            WNK::MediaPlayPause => INK::MediaPlayPause.into(),
            WNK::MediaRecord => INK::MediaRecord.into(),
            WNK::MediaRewind => INK::MediaRewind.into(),
            WNK::MediaStop => INK::MediaStop.into(),
            WNK::MediaTrackNext => INK::MediaTrackNext.into(),
            WNK::MediaTrackPrevious => INK::MediaTrackPrevious.into(),
            WNK::New => INK::New.into(),
            WNK::Open => INK::Open.into(),
            WNK::Print => INK::Print.into(),
            WNK::Save => INK::Save.into(),
            WNK::SpellCheck => INK::SpellCheck.into(),
            WNK::Key11 => INK::Key11.into(),
            WNK::Key12 => INK::Key12.into(),
            WNK::AudioBalanceLeft => INK::AudioBalanceLeft.into(),
            WNK::AudioBalanceRight => INK::AudioBalanceRight.into(),
            WNK::AudioBassBoostDown => INK::AudioBassBoostDown.into(),
            WNK::AudioBassBoostToggle => INK::AudioBassBoostToggle.into(),
            WNK::AudioBassBoostUp => INK::AudioBassBoostUp.into(),
            WNK::AudioFaderFront => INK::AudioFaderFront.into(),
            WNK::AudioFaderRear => INK::AudioFaderRear.into(),
            WNK::AudioSurroundModeNext => INK::AudioSurroundModeNext.into(),
            WNK::AudioTrebleDown => INK::AudioTrebleDown.into(),
            WNK::AudioTrebleUp => INK::AudioTrebleUp.into(),
            WNK::AudioVolumeDown => INK::AudioVolumeDown.into(),
            WNK::AudioVolumeUp => INK::AudioVolumeUp.into(),
            WNK::AudioVolumeMute => INK::AudioVolumeMute.into(),
            WNK::MicrophoneToggle => INK::MicrophoneToggle.into(),
            WNK::MicrophoneVolumeDown => INK::MicrophoneVolumeDown.into(),
            WNK::MicrophoneVolumeUp => INK::MicrophoneVolumeUp.into(),
            WNK::MicrophoneVolumeMute => INK::MicrophoneVolumeMute.into(),
            WNK::SpeechCorrectionList => INK::SpeechCorrectionList.into(),
            WNK::SpeechInputToggle => INK::SpeechInputToggle.into(),
            WNK::LaunchApplication1 => INK::LaunchApplication1.into(),
            WNK::LaunchApplication2 => INK::LaunchApplication2.into(),
            WNK::LaunchCalendar => INK::LaunchCalendar.into(),
            WNK::LaunchContacts => INK::LaunchContacts.into(),
            WNK::LaunchMail => INK::LaunchMail.into(),
            WNK::LaunchMediaPlayer => INK::LaunchMediaPlayer.into(),
            WNK::LaunchMusicPlayer => INK::LaunchMusicPlayer.into(),
            WNK::LaunchPhone => INK::LaunchPhone.into(),
            WNK::LaunchScreenSaver => INK::LaunchScreenSaver.into(),
            WNK::LaunchSpreadsheet => INK::LaunchSpreadsheet.into(),
            WNK::LaunchWebBrowser => INK::LaunchWebBrowser.into(),
            WNK::LaunchWebCam => INK::LaunchWebCam.into(),
            WNK::LaunchWordProcessor => INK::LaunchWordProcessor.into(),
            WNK::BrowserBack => INK::BrowserBack.into(),
            WNK::BrowserFavorites => INK::BrowserFavorites.into(),
            WNK::BrowserForward => INK::BrowserForward.into(),
            WNK::BrowserHome => INK::BrowserHome.into(),
            WNK::BrowserRefresh => INK::BrowserRefresh.into(),
            WNK::BrowserSearch => INK::BrowserSearch.into(),
            WNK::BrowserStop => INK::BrowserStop.into(),
            WNK::AppSwitch => INK::AppSwitch.into(),
            WNK::Call => INK::Call.into(),
            WNK::Camera => INK::Camera.into(),
            WNK::CameraFocus => INK::CameraFocus.into(),
            WNK::EndCall => INK::EndCall.into(),
            WNK::GoBack => INK::GoBack.into(),
            WNK::GoHome => INK::GoHome.into(),
            WNK::HeadsetHook => INK::HeadsetHook.into(),
            WNK::LastNumberRedial => INK::LastNumberRedial.into(),
            WNK::Notification => INK::Notification.into(),
            WNK::MannerMode => INK::MannerMode.into(),
            WNK::VoiceDial => INK::VoiceDial.into(),
            WNK::TV => INK::TV.into(),
            WNK::TV3DMode => INK::TV3DMode.into(),
            WNK::TVAntennaCable => INK::TVAntennaCable.into(),
            WNK::TVAudioDescription => INK::TVAudioDescription.into(),
            WNK::TVAudioDescriptionMixDown => INK::TVAudioDescriptionMixDown.into(),
            WNK::TVAudioDescriptionMixUp => INK::TVAudioDescriptionMixUp.into(),
            WNK::TVContentsMenu => INK::TVContentsMenu.into(),
            WNK::TVDataService => INK::TVDataService.into(),
            WNK::TVInput => INK::TVInput.into(),
            WNK::TVInputComponent1 => INK::TVInputComponent1.into(),
            WNK::TVInputComponent2 => INK::TVInputComponent2.into(),
            WNK::TVInputComposite1 => INK::TVInputComposite1.into(),
            WNK::TVInputComposite2 => INK::TVInputComposite2.into(),
            WNK::TVInputHDMI1 => INK::TVInputHDMI1.into(),
            WNK::TVInputHDMI2 => INK::TVInputHDMI2.into(),
            WNK::TVInputHDMI3 => INK::TVInputHDMI3.into(),
            WNK::TVInputHDMI4 => INK::TVInputHDMI4.into(),
            WNK::TVInputVGA1 => INK::TVInputVGA1.into(),
            WNK::TVMediaContext => INK::TVMediaContext.into(),
            WNK::TVNetwork => INK::TVNetwork.into(),
            WNK::TVNumberEntry => INK::TVNumberEntry.into(),
            WNK::TVPower => INK::TVPower.into(),
            WNK::TVRadioService => INK::TVRadioService.into(),
            WNK::TVSatellite => INK::TVSatellite.into(),
            WNK::TVSatelliteBS => INK::TVSatelliteBS.into(),
            WNK::TVSatelliteCS => INK::TVSatelliteCS.into(),
            WNK::TVSatelliteToggle => INK::TVSatelliteToggle.into(),
            WNK::TVTerrestrialAnalog => INK::TVTerrestrialAnalog.into(),
            WNK::TVTerrestrialDigital => INK::TVTerrestrialDigital.into(),
            WNK::TVTimer => INK::TVTimer.into(),
            WNK::AVRInput => INK::AVRInput.into(),
            WNK::AVRPower => INK::AVRPower.into(),
            WNK::ColorF0Red => INK::ColorF0Red.into(),
            WNK::ColorF1Green => INK::ColorF1Green.into(),
            WNK::ColorF2Yellow => INK::ColorF2Yellow.into(),
            WNK::ColorF3Blue => INK::ColorF3Blue.into(),
            WNK::ColorF4Grey => INK::ColorF4Grey.into(),
            WNK::ColorF5Brown => INK::ColorF5Brown.into(),
            WNK::ClosedCaptionToggle => INK::ClosedCaptionToggle.into(),
            WNK::Dimmer => INK::Dimmer.into(),
            WNK::DisplaySwap => INK::DisplaySwap.into(),
            WNK::DVR => INK::DVR.into(),
            WNK::Exit => INK::Exit.into(),
            WNK::FavoriteClear0 => INK::FavoriteClear0.into(),
            WNK::FavoriteClear1 => INK::FavoriteClear1.into(),
            WNK::FavoriteClear2 => INK::FavoriteClear2.into(),
            WNK::FavoriteClear3 => INK::FavoriteClear3.into(),
            WNK::FavoriteRecall0 => INK::FavoriteRecall0.into(),
            WNK::FavoriteRecall1 => INK::FavoriteRecall1.into(),
            WNK::FavoriteRecall2 => INK::FavoriteRecall2.into(),
            WNK::FavoriteRecall3 => INK::FavoriteRecall3.into(),
            WNK::FavoriteStore0 => INK::FavoriteStore0.into(),
            WNK::FavoriteStore1 => INK::FavoriteStore1.into(),
            WNK::FavoriteStore2 => INK::FavoriteStore2.into(),
            WNK::FavoriteStore3 => INK::FavoriteStore3.into(),
            WNK::Guide => INK::Guide.into(),
            WNK::GuideNextDay => INK::GuideNextDay.into(),
            WNK::GuidePreviousDay => INK::GuidePreviousDay.into(),
            WNK::Info => INK::Info.into(),
            WNK::InstantReplay => INK::InstantReplay.into(),
            WNK::Link => INK::Link.into(),
            WNK::ListProgram => INK::ListProgram.into(),
            WNK::LiveContent => INK::LiveContent.into(),
            WNK::Lock => INK::Lock.into(),
            WNK::MediaApps => INK::MediaApps.into(),
            WNK::MediaAudioTrack => INK::MediaAudioTrack.into(),
            WNK::MediaLast => INK::MediaLast.into(),
            WNK::MediaSkipBackward => INK::MediaSkipBackward.into(),
            WNK::MediaSkipForward => INK::MediaSkipForward.into(),
            WNK::MediaStepBackward => INK::MediaStepBackward.into(),
            WNK::MediaStepForward => INK::MediaStepForward.into(),
            WNK::MediaTopMenu => INK::MediaTopMenu.into(),
            WNK::NavigateIn => INK::NavigateIn.into(),
            WNK::NavigateNext => INK::NavigateNext.into(),
            WNK::NavigateOut => INK::NavigateOut.into(),
            WNK::NavigatePrevious => INK::NavigatePrevious.into(),
            WNK::NextFavoriteChannel => INK::NextFavoriteChannel.into(),
            WNK::NextUserProfile => INK::NextUserProfile.into(),
            WNK::OnDemand => INK::OnDemand.into(),
            WNK::Pairing => INK::Pairing.into(),
            WNK::PinPDown => INK::PinPDown.into(),
            WNK::PinPMove => INK::PinPMove.into(),
            WNK::PinPToggle => INK::PinPToggle.into(),
            WNK::PinPUp => INK::PinPUp.into(),
            WNK::PlaySpeedDown => INK::PlaySpeedDown.into(),
            WNK::PlaySpeedReset => INK::PlaySpeedReset.into(),
            WNK::PlaySpeedUp => INK::PlaySpeedUp.into(),
            WNK::RandomToggle => INK::RandomToggle.into(),
            WNK::RcLowBattery => INK::RcLowBattery.into(),
            WNK::RecordSpeedNext => INK::RecordSpeedNext.into(),
            WNK::RfBypass => INK::RfBypass.into(),
            WNK::ScanChannelsToggle => INK::ScanChannelsToggle.into(),
            WNK::ScreenModeNext => INK::ScreenModeNext.into(),
            WNK::Settings => INK::Settings.into(),
            WNK::SplitScreenToggle => INK::SplitScreenToggle.into(),
            WNK::STBInput => INK::STBInput.into(),
            WNK::STBPower => INK::STBPower.into(),
            WNK::Subtitle => INK::Subtitle.into(),
            WNK::Teletext => INK::Teletext.into(),
            WNK::VideoModeNext => INK::VideoModeNext.into(),
            WNK::Wink => INK::Wink.into(),
            WNK::ZoomToggle => INK::ZoomToggle.into(),
            WNK::F1 => INK::F1.into(),
            WNK::F2 => INK::F2.into(),
            WNK::F3 => INK::F3.into(),
            WNK::F4 => INK::F4.into(),
            WNK::F5 => INK::F5.into(),
            WNK::F6 => INK::F6.into(),
            WNK::F7 => INK::F7.into(),
            WNK::F8 => INK::F8.into(),
            WNK::F9 => INK::F9.into(),
            WNK::F10 => INK::F10.into(),
            WNK::F11 => INK::F11.into(),
            WNK::F12 => INK::F12.into(),
            WNK::F13 => INK::F13.into(),
            WNK::F14 => INK::F14.into(),
            WNK::F15 => INK::F15.into(),
            WNK::F16 => INK::F16.into(),
            WNK::F17 => INK::F17.into(),
            WNK::F18 => INK::F18.into(),
            WNK::F19 => INK::F19.into(),
            WNK::F20 => INK::F20.into(),
            WNK::F21 => INK::F21.into(),
            WNK::F22 => INK::F22.into(),
            WNK::F23 => INK::F23.into(),
            WNK::F24 => INK::F24.into(),
            WNK::F25 => INK::F25.into(),
            WNK::F26 => INK::F26.into(),
            WNK::F27 => INK::F27.into(),
            WNK::F28 => INK::F28.into(),
            WNK::F29 => INK::F29.into(),
            WNK::F30 => INK::F30.into(),
            WNK::F31 => INK::F31.into(),
            WNK::F32 => INK::F32.into(),
            WNK::F33 => INK::F33.into(),
            WNK::F34 => INK::F34.into(),
            WNK::F35 => INK::F35.into(),
            _ => input::Key::Unknown(None),
        }
    }
}

impl From<winit::keyboard::NativeKey> for input::Key {
    fn from(key: winit::keyboard::NativeKey) -> Self {
        match key {
            winit::keyboard::NativeKey::Unidentified => input::Key::Unknown(None),
            winit::keyboard::NativeKey::Android(code) => input::Key::Unknown(Some(code)),
            winit::keyboard::NativeKey::MacOS(code) => input::Key::Unknown(Some(code as u32)),
            winit::keyboard::NativeKey::Windows(code) => input::Key::Unknown(Some(code as u32)),
            winit::keyboard::NativeKey::Xkb(code) => input::Key::Unknown(Some(code)),
            winit::keyboard::NativeKey::Web(c) => input::Key::Character(
                // TODO: remove this conversion as soon as winit updates
                c.as_str().into(),
            ),
        }
    }
}

impl From<winit::keyboard::Key> for input::Key {
    fn from(key: winit::keyboard::Key) -> Self {
        match key {
            winit::keyboard::Key::Named(key) => key.into(),
            winit::keyboard::Key::Character(c) => input::Key::Character(
                // TODO: remove this conversion as soon as winit updates
                c.as_str().into(),
            ),
            winit::keyboard::Key::Unidentified(key) => key.into(),
            winit::keyboard::Key::Dead(c) => input::Key::Dead(c),
        }
    }
}

impl From<winit::keyboard::KeyLocation> for input::KeyLocation {
    #[inline]
    fn from(location: winit::keyboard::KeyLocation) -> Self {
        match location {
            winit::keyboard::KeyLocation::Standard => input::KeyLocation::Standard,
            winit::keyboard::KeyLocation::Left => input::KeyLocation::Left,
            winit::keyboard::KeyLocation::Right => input::KeyLocation::Right,
            winit::keyboard::KeyLocation::Numpad => input::KeyLocation::Numpad,
        }
    }
}

impl From<winit::event::KeyEvent> for input::InputEvent {
    fn from(event: winit::event::KeyEvent) -> Self {
        match event.state {
            winit::event::ElementState::Pressed => input::InputEvent::KeyPressed {
                key: event.logical_key.into(),
                location: event.location.into(),
                // TODO: remove this conversion as soon as winit updates
                text: event.text.map(|text| text.as_str().into()),
                repeat: event.repeat,
            },
            winit::event::ElementState::Released => input::InputEvent::KeyReleased {
                key: event.logical_key.into(),
                location: event.location.into(),
                // TODO: remove this conversion as soon as winit updates
                text: event.text.map(|text| text.as_str().into()),
            },
        }
    }
}
