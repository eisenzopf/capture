use cocoa::appkit::{NSApplication, NSMenu, NSMenuItem};
use cocoa::base::{id, nil, YES};
use cocoa::foundation::{NSAutoreleasePool, NSString};
use objc::{class, msg_send, sel, sel_impl, declare::ClassDecl, runtime::{Object, Sel}};
use std::cell::RefCell;
use core_foundation::bundle::CFBundle;

mod audio_recorder;
use audio_recorder::AudioRecorder;

thread_local! {
    static RECORDER: RefCell<Option<AudioRecorder>> = RefCell::new(None);
    static RECORDING_STATE: RefCell<bool> = RefCell::new(false);
}

unsafe fn check_microphone_permission() -> bool {
    //println!("Checking microphone permissions...");
    
    let bundle = CFBundle::main_bundle();
    //println!("Got main bundle");
    
    // Just check if we got the dictionary
    let _info_dict = bundle.info_dictionary();
    //println!("Running from app bundle with info dictionary");
    
    true
}

extern fn menu_action(_this: &Object, _cmd: Sel, item: id) {
    unsafe {
        let current_title: id = msg_send![item, title];
        let title_str: *const i8 = msg_send![current_title, UTF8String];
        let is_recording = std::ffi::CStr::from_ptr(title_str)
            .to_string_lossy()
            .contains("Start");

        //println!("Menu action triggered. Is recording: {}", is_recording);

        RECORDER.with(|recorder| {
            if let Some(recorder) = &mut *recorder.borrow_mut() {
                if is_recording {
                    //println!("Attempting to start recording...");
                    match recorder.start_recording() {
                        Ok(_) => {
                            //println!("Recording started successfully");
                            RECORDING_STATE.with(|state| {
                                *state.borrow_mut() = true;
                            });
                            
                            // Update menu item
                            let stop_title = NSString::alloc(nil).init_str("Stop Recording");
                            let _: () = msg_send![item, setTitle:stop_title];
                            
                            // Update icon to filled circle
                            let menu: id = msg_send![item, menu];
                            let status_item: id = msg_send![menu, delegate];
                            let button: id = msg_send![status_item, button];
                            
                            let image: id = msg_send![class!(NSImage), imageNamed:NSString::alloc(nil).init_str("NSStatusAvailable")];
                            let _: () = msg_send![button, setImage:image];
                        },
                        Err(e) => {
                            println!("Failed to start recording: {:?}", e);
                        }
                    }
                } else {
                    //println!("Stopping recording...");
                    recorder.stop_recording();
                    RECORDING_STATE.with(|state| {
                        *state.borrow_mut() = false;
                    });
                    
                    // Update menu item
                    let start_title = NSString::alloc(nil).init_str("Start Recording");
                    let _: () = msg_send![item, setTitle:start_title];
                    
                    // Update icon back to microphone
                    let menu: id = msg_send![item, menu];
                    let status_item: id = msg_send![menu, delegate];
                    let button: id = msg_send![status_item, button];
                    
                    let image: id = msg_send![class!(NSImage), imageNamed:NSString::alloc(nil).init_str("MenuBarMicIcon")];
                    let _: () = msg_send![button, setImage:image];
                }
            }
        });
    }
}

fn register_custom_image() -> id {
    unsafe {
        let image_name = NSString::alloc(nil).init_str("MenuBarMicIcon");
        
        // Create SVG data for a simple microphone icon
        // Reduced viewBox and width/height to 18px (standard menu bar size)
        let svg_data = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
        <svg width=\"18px\" height=\"18px\" viewBox=\"0 0 18 18\" version=\"1.1\" xmlns=\"http://www.w3.org/2000/svg\">\
            <g stroke=\"none\" stroke-width=\"1\" fill=\"none\" fill-rule=\"evenodd\">\
                <path d=\"M9,2 C10.3807119,2 11.5,3.11928813 11.5,4.5 L11.5,9 C11.5,10.3807119 10.3807119,11.5 9,11.5 C7.61928813,11.5 6.5,10.3807119 6.5,9 L6.5,4.5 C6.5,3.11928813 7.61928813,2 9,2 Z M9,12.5 C11.2091391,12.5 13,10.7091391 13,8.5 L13,8 L14,8 L14,8.5 C14,11.2316253 11.9866438,13.5726874 9.33315164,13.9538371 L9.33333333,16 L8.66666667,16 L8.66684836,13.9538371 C6.01335623,13.5726874 4,11.2316253 4,8.5 L4,8 L5,8 L5,8.5 C5,10.7091391 6.79086089,12.5 9,12.5 Z\" fill=\"#000000\" fill-rule=\"nonzero\"/>\
            </g>\
        </svg>";

        let data: id = msg_send![class!(NSData), dataWithBytes:svg_data.as_ptr() length:svg_data.len()];
        let image: id = msg_send![class!(NSImage), alloc];
        let _: () = msg_send![image, initWithData:data];
        let _: () = msg_send![image, setTemplate:YES];
        let _: () = msg_send![image, setName:image_name];

        image
    }
}

struct AppDelegate {
    status_item: id,
}

impl AppDelegate {
    fn new() -> Self {
        AppDelegate {
            status_item: nil,
        }
    }

    unsafe fn setup_menu_bar(&mut self) {
        //println!("Setting up menu bar...");
        
        // Check permissions first
        check_microphone_permission();
        
        // Initialize recorder
        RECORDER.with(|r| {
            //println!("Initializing audio recorder...");
            *r.borrow_mut() = Some(AudioRecorder::new());
        });

        // Register the custom microphone icon
        //println!("Registering custom icon...");
        register_custom_image();

        // Set up status bar item
        let status_bar: id = msg_send![class!(NSStatusBar), systemStatusBar];
        let status_item: id = msg_send![status_bar, statusItemWithLength:-1.0];
        self.status_item = status_item;
        
        let button: id = msg_send![status_item, button];
        
        // Set up initial microphone icon
        let image: id = msg_send![class!(NSImage), imageNamed:NSString::alloc(nil).init_str("MenuBarMicIcon")];
        let _: () = msg_send![button, setImage:image];
        
        // Create menu
        let menu = NSMenu::new(nil).autorelease();
        let _: () = msg_send![menu, setDelegate:status_item];
        
        // Create Record/Stop item
        let record_item = NSMenuItem::new(nil).autorelease();
        let record_title = NSString::alloc(nil).init_str("Start Recording");
        let _: () = msg_send![record_item, setTitle:record_title];
        let _: () = msg_send![record_item, setEnabled:YES];
        
        // Register the action method
        let superclass = class!(NSObject);
        let mut decl = ClassDecl::new("RecorderDelegate", superclass).unwrap();
        decl.add_method(
            sel!(menuAction:),
            menu_action as extern fn(&Object, Sel, id)
        );
        let delegate_class = decl.register();
        let delegate: id = msg_send![delegate_class, new];
        
        // Set up action
        let _: () = msg_send![record_item, setTarget:delegate];
        let _: () = msg_send![record_item, setAction:sel!(menuAction:)];
        
        // Create Quit item
        let quit_item = NSMenuItem::new(nil).autorelease();
        let quit_title = NSString::alloc(nil).init_str("Quit");
        let _: () = msg_send![quit_item, setTitle:quit_title];
        let _: () = msg_send![quit_item, setEnabled:YES];
        let _: () = msg_send![quit_item, setAction:sel!(terminate:)];
        
        // Add items to menu
        menu.addItem_(record_item);
        menu.addItem_(quit_item);
        
        let _: () = msg_send![status_item, setMenu:menu];
        
        // Retain the delegate
        let _: () = msg_send![delegate, retain];
        
        //println!("Menu bar setup complete");
    }
}

fn main() {
    //println!("Starting application...");
    
    unsafe {
        let _pool = NSAutoreleasePool::new(nil);
        let app = NSApplication::sharedApplication(nil);
        let mut delegate = AppDelegate::new();
        
        //println!("Setting up menu bar...");
        delegate.setup_menu_bar();
        
        //println!("Running application...");
        app.run();
    }
}