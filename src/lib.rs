extern crate xcb;
use std::collections::HashMap;
use std::error::Error;
use std::u32;

pub struct ClipboardManager {
    conn: xcb::Connection,
    atoms: HashMap<&'static str, xcb::Atom>,
    data: String,  // TODO: Do lifetime here with &str (maybe?)
    test: bool
}

impl ClipboardManager {
    pub fn new(data: String, test: bool) -> ClipboardManager {
        // Create a Connection to x11
        let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
        // Asks server for the connection data
        let setup = conn.get_setup();
        // Get root window
        let root = setup.roots().nth(screen_num as usize).unwrap();
        // Generate xid for the connection
        let window = conn.generate_id();

        // Events we want to react to
        let selection_values = [
            (xcb::CW_EVENT_MASK, xcb::EVENT_MASK_STRUCTURE_NOTIFY),
        ];

        // Create new window as child from the root window
        // The window won't show up, but the window size needs to be greater than 1 !!!
        // Wasted much time because everything was set to 0
        xcb::create_window_checked(
            &conn,
            xcb::COPY_FROM_PARENT as u8,
            window,
            root.root(), 
            0,
            0,
            1,
            1,
            0,
            xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
            root.root_visual(),
            &selection_values
        );

        // Atoms we need
        // CLIPBOARD = ctrl+v
        // PRIMARY = middle mouse button
        let clipboard = xcb::intern_atom(&conn, false, "CLIPBOARD").get_reply().unwrap().atom();
        let primary = xcb::intern_atom(&conn, false, "PRIMARY").get_reply().unwrap().atom();
        // The targets we (hopefully) support
        let utf8 = xcb::intern_atom(&conn, false, "UTF8_STRING").get_reply().unwrap().atom();
        let targets = xcb::intern_atom(&conn, false, "TARGETS").get_reply().unwrap().atom();
        let multiple = xcb::intern_atom(&conn, false, "MULTIPLE").get_reply().unwrap().atom();
        let text_utf8 = xcb::intern_atom(&conn, false, "text/plain;charset=utf-8").get_reply().unwrap().atom();
        let text_gtk = xcb::intern_atom(&conn, false, "GTK_TEXT_BUFFER_CONTENTS").get_reply().unwrap().atom(); // Read into this, not quite right yet

        // Take ownership of the clipboard and primary selection
        xcb::set_selection_owner_checked(
            &conn,
            window,
            clipboard,
            xcb::CURRENT_TIME
        );

        xcb::set_selection_owner_checked(
            &conn,
            window,
            primary,
            xcb::CURRENT_TIME
        );

        // Flush for good measure
        conn.flush();

        // Create hashmap with atoms
        let mut atoms = HashMap::new();
        atoms.insert("clipboard", clipboard);
        atoms.insert("primary", primary);
        atoms.insert("utf8", utf8);
        atoms.insert("targets", targets);
        atoms.insert("multiple", multiple);
        atoms.insert("text_utf8", text_utf8);
        atoms.insert("text_gtk", text_gtk);

        ClipboardManager {
            conn,
            atoms,
            data,
            test
        }
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        // As long we don't run into an error we wait for the next event
        while let Some(event) = self.conn.wait_for_event() {
            match event.response_type() {
                // If another program takes ownership
                xcb::SELECTION_CLEAR => {
                    break;
                },
                // If a program requests the clipboard
                xcb::SELECTION_REQUEST => {
                    // Cast the event into an selectionrequestevent, which contains the caller informations
                    let req: &xcb::SelectionRequestEvent = unsafe {
                        xcb::cast_event(&event)
                    };

                    // Change the requested property
                    // Only care about the conversation if the property is not none
                    if req.property() != xcb::NONE {
                        self.change_property(&req);
                    }

                    // Create a new notify event
                    // It will be send to the caller to notify it of the change
                    let notify = xcb::SelectionNotifyEvent::new(
                        xcb::CURRENT_TIME,
                        req.requestor(),
                        req.selection(),
                        req.target(),
                        req.property()
                    );

                    // Send it to the caller (notify the caller of the change)
                    xcb::send_event(
                        &self.conn,
                        false,
                        req.requestor(),
                        xcb::EVENT_MASK_PROPERTY_CHANGE,
                        &notify
                    );

                    // Flush
                    self.conn.flush();

                },
                _ => ()
            };
        }
       Ok(()) 
    }

    fn change_property(&self, req: &xcb::SelectionRequestEvent) {
        // Debugging
        if self.test {
            println!("Target {} requested", xcb::get_atom_name(&self.conn, req.target().clone()).get_reply().unwrap().name());
        }

        // Check for different properties and act accordingly
        if req.target() == self.atoms["targets"] {
            // targets_atom should return a list of all avaiable atoms
            let available_atoms: Vec<xcb::Atom> = self.atoms.values().map(|x| x.clone()).collect();
            xcb::change_property_checked(
                &self.conn,
                xcb::PROP_MODE_REPLACE as u8,
                req.requestor(),
                req.property(),
                req.target(),
                32 as u8, // Read more into this, seems important (?)
                &available_atoms
            );
        } else if req.target() == self.atoms["multiple"] {
            // The caller send multiple targets and properties it wants to get changed
            
            // List of atom pairs
            // 1: target
            // 2: property to convert the target to
            // If we can't convert the target, property should be set to None
            
            // This should give us the vector of those atom pairs(?)
            // TODO: Find program which actually uses the MULTIPLE target
            // I know that it's not implemented yet
            let atom_pairs: Vec<u32> = xcb::get_property(
                &self.conn,
                false,
                req.requestor(),
                req.property(),
                req.target(),
                0,
                u32::MAX
            ).get_reply()
            .unwrap()
            .value()
            .to_owned();

            if self.test {
                println!("Found atom pairs: {:?}", &atom_pairs);
            }

        } else if req.target() == self.atoms["utf8"] || req.target() == self.atoms["text_utf8"] || req.target() == self.atoms["text_gtk"] {
            // Normal utf8 request
            xcb::change_property_checked(
                &self.conn,
                xcb::PROP_MODE_REPLACE as u8,
                req.requestor(),
                req.property(),
                req.target(),
                8 as u8, // Read more into this, seems important (?)
                self.data.as_bytes()
            );
        } else {
            //Unknown target (for debugging)
            if self.test {
                println!("Unknown target {} requested", xcb::get_atom_name(&self.conn, req.target().clone()).get_reply().unwrap().name());
            }
        }
    }
}