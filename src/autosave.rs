use std::rc::Rc;
use std::cell::RefCell;
use std::time::Duration;
use crate::app::AppState;

pub fn schedule_autosave(app_state: Rc<RefCell<AppState>>) {
    let mut state = app_state.borrow_mut();
    
    if let Some(timer_id) = state.save_timer.take() {
        timer_id.remove();
    }
    
    let delay_ms = state.config.autosave.delay_ms;
    let app_state_clone = app_state.clone();
    
    let source_id = glib::timeout_add_local_once(
        Duration::from_millis(delay_ms),
        move || {
            if let Ok(mut state) = app_state_clone.try_borrow_mut() {
                state.save_timer = None;
                if let Err(e) = state.save_current_document() {
                    eprintln!("Autosave failed: {:?}", e);
                    state.show_error(&format!("Autosave failed: {}", e));
                }
            }
        },
    );
    
    state.save_timer = Some(source_id);
}
