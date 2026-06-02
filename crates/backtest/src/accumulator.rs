// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

//! Time event accumulation and scheduling for the backtest engine.

use std::{cmp::Reverse, collections::BinaryHeap};

use nautilus_common::{clock::TestClock, timer::TimeEventHandler};
use nautilus_core::UnixNanos;

/// Provides a means of accumulating and draining time event handlers using a priority queue.
///
/// Events are maintained in timestamp order using a binary heap, allowing efficient
/// retrieval of the next event to process.
#[derive(Debug)]
pub struct TimeEventAccumulator {
    heap: BinaryHeap<Reverse<TimeEventHandler>>,
}

impl TimeEventAccumulator {
    /// Creates a new [`TimeEventAccumulator`] instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }

    /// Advance the given clock to the `to_time_ns` and push events to the heap.
    pub fn advance_clock(&mut self, clock: &mut TestClock, to_time_ns: UnixNanos, set_time: bool) {
        let events = clock.advance_time(to_time_ns, set_time);
        let handlers = clock.match_handlers(events);
        for handler in handlers {
            self.heap.push(Reverse(handler));
        }
    }

    /// Peek at the next event timestamp without removing it.
    ///
    /// Returns `None` if the heap is empty.
    #[must_use]
    pub fn peek_next_time(&self) -> Option<UnixNanos> {
        self.heap.peek().map(|h| h.0.event.ts_event)
    }

    /// Pop the next event if its timestamp is at or before `ts`.
    ///
    /// Returns `None` if the heap is empty or the next event is after `ts`.
    pub fn pop_next_at_or_before(&mut self, ts: UnixNanos) -> Option<TimeEventHandler> {
        if self.heap.peek().is_some_and(|h| h.0.event.ts_event <= ts) {
            self.heap.pop().map(|h| h.0)
        } else {
            None
        }
    }

    /// Check if the heap is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// Get the number of events in the heap.
    #[must_use]
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    /// Clear all events from the heap.
    pub fn clear(&mut self) {
        self.heap.clear();
    }

    /// Drain all events from the heap in timestamp order.
    ///
    /// This is provided for backwards compatibility with code that expects
    /// batch processing. For iterative processing, prefer `pop_next_at_or_before`.
    pub fn drain(&mut self) -> Vec<TimeEventHandler> {
        let mut handlers = Vec::with_capacity(self.heap.len());
        while let Some(scheduled) = self.heap.pop() {
            handlers.push(scheduled.0);
        }
        handlers
    }
}

impl Default for TimeEventAccumulator {
    /// Creates a new default [`TimeEventAccumulator`] instance.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use nautilus_common::timer::{TimeEvent, TimeEventCallback};
    use nautilus_core::UUID4;
    use rstest::*;
    use ustr::Ustr;

    use super::*;

    fn noop_callback() -> TimeEventCallback {
        TimeEventCallback::from(|_: TimeEvent| {})
    }

    fn test_event(name: &str, ts: u64) -> TimeEvent {
        TimeEvent::new(Ustr::from(name), UUID4::new(), ts.into(), ts.into())
    }

    #[rstest]
    fn test_accumulator_pop_in_order() {
        let mut accumulator = TimeEventAccumulator::new();
        let callback = noop_callback();

        let time_event1 = test_event("TEST_EVENT_1", 100);
        let time_event2 = test_event("TEST_EVENT_2", 300);
        let time_event3 = test_event("TEST_EVENT_3", 200);

        accumulator.heap.push(Reverse(TimeEventHandler::new(
            time_event1.clone(),
            callback.clone(),
        )));
        accumulator.heap.push(Reverse(TimeEventHandler::new(
            time_event2.clone(),
            callback.clone(),
        )));
        accumulator.heap.push(Reverse(TimeEventHandler::new(
            time_event3.clone(),
            callback,
        )));
        assert_eq!(accumulator.len(), 3);

        let popped1 = accumulator.pop_next_at_or_before(1000.into()).unwrap();
        assert_eq!(popped1.event.ts_event, time_event1.ts_event);

        let popped2 = accumulator.pop_next_at_or_before(1000.into()).unwrap();
        assert_eq!(popped2.event.ts_event, time_event3.ts_event);

        let popped3 = accumulator.pop_next_at_or_before(1000.into()).unwrap();
        assert_eq!(popped3.event.ts_event, time_event2.ts_event);

        assert!(accumulator.is_empty());
    }

    #[rstest]
    fn test_accumulator_pop_same_timestamp_in_name_order() {
        let mut accumulator = TimeEventAccumulator::new();
        let callback = noop_callback();

        let spread_event = test_event("SPREAD_QUOTE_ESM4", 100);
        let time_bar_event = test_event("TIME_BAR_ESM4-2-MINUTE-ASK-INTERNAL", 100);

        accumulator.heap.push(Reverse(TimeEventHandler::new(
            time_bar_event.clone(),
            callback.clone(),
        )));
        accumulator.heap.push(Reverse(TimeEventHandler::new(
            spread_event.clone(),
            callback,
        )));

        let popped1 = accumulator.pop_next_at_or_before(100.into()).unwrap();
        assert_eq!(popped1.event.ts_event, spread_event.ts_event);
        assert_eq!(popped1.event.name, spread_event.name);

        let popped2 = accumulator.pop_next_at_or_before(100.into()).unwrap();
        assert_eq!(popped2.event.ts_event, time_bar_event.ts_event);
        assert_eq!(popped2.event.name, time_bar_event.name);
    }

    #[rstest]
    fn test_accumulator_pop_respects_timestamp() {
        let mut accumulator = TimeEventAccumulator::new();
        let callback = noop_callback();

        let time_event1 = test_event("TEST_EVENT_1", 100);
        let time_event2 = test_event("TEST_EVENT_2", 300);

        accumulator.heap.push(Reverse(TimeEventHandler::new(
            time_event1.clone(),
            callback.clone(),
        )));
        accumulator.heap.push(Reverse(TimeEventHandler::new(
            time_event2.clone(),
            callback,
        )));

        let popped1 = accumulator.pop_next_at_or_before(200.into()).unwrap();
        assert_eq!(popped1.event.ts_event, time_event1.ts_event);

        // Event at 300 should not be returned with ts=200.
        assert!(accumulator.pop_next_at_or_before(200.into()).is_none());

        let popped2 = accumulator.pop_next_at_or_before(300.into()).unwrap();
        assert_eq!(popped2.event.ts_event, time_event2.ts_event);
    }

    #[rstest]
    fn test_peek_next_time() {
        let mut accumulator = TimeEventAccumulator::new();
        assert!(accumulator.peek_next_time().is_none());

        let time_event1 = test_event("TEST_EVENT_1", 200);
        let time_event2 = test_event("TEST_EVENT_2", 100);
        let callback = noop_callback();

        accumulator.heap.push(Reverse(TimeEventHandler::new(
            time_event1,
            callback.clone(),
        )));
        assert_eq!(accumulator.peek_next_time(), Some(200.into()));

        accumulator
            .heap
            .push(Reverse(TimeEventHandler::new(time_event2, callback)));
        assert_eq!(accumulator.peek_next_time(), Some(100.into()));
    }

    #[rstest]
    fn test_drain_returns_in_order() {
        let mut accumulator = TimeEventAccumulator::new();
        let callback = noop_callback();

        for ts in [300_u64, 100, 200] {
            accumulator.heap.push(Reverse(TimeEventHandler::new(
                test_event("TEST", ts),
                callback.clone(),
            )));
        }

        let handlers = accumulator.drain();

        assert_eq!(handlers.len(), 3);
        assert_eq!(handlers[0].event.ts_event.as_u64(), 100);
        assert_eq!(handlers[1].event.ts_event.as_u64(), 200);
        assert_eq!(handlers[2].event.ts_event.as_u64(), 300);
        assert!(accumulator.is_empty());
    }

    #[rstest]
    fn test_interleaved_push_pop_maintains_order() {
        let mut accumulator = TimeEventAccumulator::new();
        let callback = noop_callback();
        let mut popped_timestamps: Vec<u64> = Vec::new();

        for ts in [100_u64, 300] {
            accumulator.heap.push(Reverse(TimeEventHandler::new(
                test_event("TEST", ts),
                callback.clone(),
            )));
        }

        let handler = accumulator.pop_next_at_or_before(1000.into()).unwrap();
        popped_timestamps.push(handler.event.ts_event.as_u64());

        // Simulate a callback scheduling a new event between the popped and pending events.
        accumulator.heap.push(Reverse(TimeEventHandler::new(
            test_event("NEW", 150),
            callback,
        )));

        while let Some(handler) = accumulator.pop_next_at_or_before(1000.into()) {
            popped_timestamps.push(handler.event.ts_event.as_u64());
        }

        assert_eq!(popped_timestamps, vec![100, 150, 300]);
    }

    #[rstest]
    fn test_same_timestamp_events() {
        let mut accumulator = TimeEventAccumulator::new();
        let callback = noop_callback();

        for i in 0..3 {
            accumulator.heap.push(Reverse(TimeEventHandler::new(
                test_event(&format!("EVENT_{i}"), 100),
                callback.clone(),
            )));
        }

        let mut count = 0;
        while let Some(handler) = accumulator.pop_next_at_or_before(100.into()) {
            assert_eq!(handler.event.ts_event.as_u64(), 100);
            count += 1;
        }
        assert_eq!(count, 3);
    }

    #[rstest]
    fn test_pop_at_exact_timestamp_boundary() {
        let mut accumulator = TimeEventAccumulator::new();
        let callback = noop_callback();

        accumulator.heap.push(Reverse(TimeEventHandler::new(
            test_event("TEST", 100),
            callback.clone(),
        )));

        let handler = accumulator.pop_next_at_or_before(100.into());
        assert!(handler.is_some());
        assert_eq!(handler.unwrap().event.ts_event.as_u64(), 100);

        accumulator.heap.push(Reverse(TimeEventHandler::new(
            test_event("TEST2", 200),
            callback,
        )));

        assert!(accumulator.pop_next_at_or_before(199.into()).is_none());
        assert!(accumulator.pop_next_at_or_before(200.into()).is_some());
    }
}
