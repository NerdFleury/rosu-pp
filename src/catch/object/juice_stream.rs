use std::vec::Drain;

use rosu_map::section::hit_objects::{
    CurveBuffers, PathControlPoint, SliderEvent, SliderEventType, SliderEventsIter,
};

use crate::{
    catch::{attributes::CatchDifficultyAttributesBuilder, convert::CatchBeatmap},
    model::{
        control_point::{DifficultyPoint, TimingPoint},
        hit_object::Slider,
    },
};

pub struct JuiceStream<'a> {
    pub control_points: &'a [PathControlPoint], // needed for applying hr offset
    pub nested_objects: Drain<'a, NestedJuiceStreamObject>,
}

impl<'a> JuiceStream<'a> {
    pub const BASE_SCORING_DIST: f64 = 100.0;

    pub fn new(
        x: f32,
        start_time: f64,
        slider: &'a Slider,
        converted: &CatchBeatmap<'_>,
        attrs: &mut CatchDifficultyAttributesBuilder,
        bufs: &'a mut JuiceStreamBufs,
    ) -> Self {
        let slider_multiplier = converted.map.slider_multiplier;
        let slider_tick_rate = converted.map.slider_tick_rate;

        let beat_len = converted
            .map
            .timing_point_at(start_time)
            .map_or(TimingPoint::DEFAULT_BEAT_LEN, |point| point.beat_len);

        let slider_velocity = converted
            .map
            .difficulty_point_at(start_time)
            .map_or(DifficultyPoint::DEFAULT_SLIDER_VELOCITY, |point| {
                point.slider_velocity
            });

        let path = slider.curve(&mut bufs.curve);

        let velocity_factor = JuiceStream::BASE_SCORING_DIST * slider_multiplier / beat_len;
        let velocity = velocity_factor * slider_velocity;
        let tick_dist_factor =
            JuiceStream::BASE_SCORING_DIST * slider_multiplier / slider_tick_rate;

        let tick_dist_multiplier = if converted.map.version < 8 {
            1.0
        } else {
            slider_velocity
        };

        let tick_dist = tick_dist_factor * tick_dist_multiplier;

        let span_count = slider.span_count() as f64;
        let duration = span_count * path.dist() / velocity;
        let span_duration = duration / span_count;

        let events = SliderEventsIter::new(
            start_time,
            span_duration,
            velocity,
            tick_dist,
            path.dist(),
            slider.span_count() as i32,
            &mut bufs.ticks,
        );

        let mut last_event_time = None;

        for e in events {
            if let Some(last_event_time) = last_event_time.filter(|_| attrs.take_more()) {
                let since_last_tick = e.time - last_event_time;

                if since_last_tick > 80.0 {
                    let mut time_between_tiny = since_last_tick;

                    while time_between_tiny > 100.0 {
                        time_between_tiny /= 2.0;
                    }

                    let mut t = time_between_tiny;

                    while t < since_last_tick {
                        attrs.inc_tiny_droplets();

                        let nested = NestedJuiceStreamObject {
                            pos: 0.0,        // not important
                            start_time: 0.0, // not important
                            kind: NestedJuiceStreamObjectKind::TinyDroplet,
                        };

                        bufs.nested_objects.push(nested);

                        t += time_between_tiny;
                    }
                }
            }

            last_event_time = Some(e.time);

            let kind = match e.kind {
                SliderEventType::Tick => {
                    attrs.inc_droplets();

                    NestedJuiceStreamObjectKind::Droplet
                }
                SliderEventType::Head | SliderEventType::Repeat | SliderEventType::Tail => {
                    attrs.inc_fruits();

                    NestedJuiceStreamObjectKind::Fruit
                }
                SliderEventType::LastTick => continue,
            };

            let nested = NestedJuiceStreamObject {
                pos: x + path.position_at(e.path_progress).x,
                start_time: e.time,
                kind,
            };

            bufs.nested_objects.push(nested);
        }

        Self {
            control_points: slider.control_points.as_ref(),
            nested_objects: bufs.nested_objects.drain(..),
        }
    }
}

pub struct NestedJuiceStreamObject {
    pub pos: f32,
    pub start_time: f64,
    pub kind: NestedJuiceStreamObjectKind,
}

pub enum NestedJuiceStreamObjectKind {
    Fruit,
    Droplet,
    TinyDroplet,
}

pub struct JuiceStreamBufs {
    pub nested_objects: Vec<NestedJuiceStreamObject>,
    pub curve: CurveBuffers,
    pub ticks: Vec<SliderEvent>,
}
