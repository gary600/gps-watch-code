//! State machine stuff

use embedded_graphics::Drawable;

/// State shared by the different UI modes
struct SharedState {

}

/// A single UI screen, such as the clock or the stopwatch.
trait Screen {
    /// Called when the screen is switched to
    fn enter(&mut self, shared_state: &mut SharedState);
    /// Called on a certain frequency
    fn update(&mut self, shared_state: &mut SharedState);
    /// Called when a redraw is needed
    fn draw<D: Drawable>(&self, shared_state: &SharedState, display: &mut D);
    /// Called when the screen is switched away from
    fn exit(&mut self, shared_state: &mut SharedState);
}