extern crate gstreamer as gst;
use gst::prelude::*;

fn main() {
    gst::init().unwrap();
    let main_loop = glib::MainLoop::new(None, false);

    let input_pipeline = gst::Pipeline::new("file-pipeline");
    let output_pipeline = gst::Pipeline::new("output-pipeline");
    let uri = "file:///e:/test.mp4";
    let uridecodebin = gst::ElementFactory::make("uridecodebin", None).unwrap();
    uridecodebin.set_property("uri", &uri).unwrap();
    let videoconvert = gst::ElementFactory::make("videoconvert", None).unwrap();
    let videorate = gst::ElementFactory::make("videorate", None).unwrap();
    let videoscale = gst::ElementFactory::make("videoscale", None).unwrap();
    let proxy_sink = gst::ElementFactory::make("intervideosink", None).unwrap();
    let proxy_src = gst::ElementFactory::make("intervideosrc", None).unwrap();
    //proxy_src.set_property("proxysink", &proxy_sink).unwrap();
    let videosink = gst::ElementFactory::make("autovideosink", None).unwrap();

    input_pipeline.add_many(&[&uridecodebin, &videoconvert, &videorate, &videoscale, &proxy_sink]).unwrap();
    gst::Element::link_many(&[&videoconvert, &videorate, &videoscale, &proxy_sink]).unwrap();
    output_pipeline.add_many(&[&proxy_src, &videosink]).unwrap();
    gst::Element::link_many(&[&proxy_src, &videosink]).unwrap();

    let videoconvert_weak = videoconvert.downgrade();
    uridecodebin.connect_pad_added(move |_, src_pad| {
        let new_pad_caps = src_pad
                .get_current_caps()
                .expect("Failed to get caps of new pad.");
            let new_pad_struct = new_pad_caps
                .get_structure(0)
                .expect("Failed to get first structure of caps.");
            let new_pad_type = new_pad_struct.get_name();

            if new_pad_type.starts_with("video/x-raw") {
                let videoconvert = match videoconvert_weak.upgrade() {
                    Some(videoconvert) => videoconvert,
                    None => return,
                };

                let sink_pad = videoconvert
                    .get_static_pad("sink")
                    .expect("Failed to get static sink pad from videoconvert");
                if sink_pad.is_linked() {
                    println!("We are already linked. Ignoring.");
                    return;
                }
                src_pad.link(&sink_pad).unwrap();
            }
    });

    output_pipeline.set_state(gst::State::Playing).unwrap();
    input_pipeline.set_state(gst::State::Playing).unwrap();

    let bus = input_pipeline.get_bus().unwrap();
    let input_pipeline_weak = input_pipeline.downgrade();
    let mut did_cue = false;
    bus.add_watch(move |_, msg| {
            use crate::gst::MessageView;
            match msg.view() {
                MessageView::StateChanged(state_changed) => {
                    let input_pipeline = match input_pipeline_weak.upgrade() {
                        Some(input_pipeline) => input_pipeline,
                        None => return glib::Continue(true),
                    };
                    if state_changed
                        .get_src()
                        .map(|s| s == input_pipeline)
                        .unwrap_or(false)
                    {
                        println!(
                            "Pipeline state changed from {:?} to {:?}",
                            state_changed.get_old(),
                            state_changed.get_current()
                        );
                        let new_state = state_changed.get_current();
                        if (new_state ==  gst::State::Playing && did_cue == false) {
                            println!("DO The Cue CUE!");
                            input_pipeline.seek_simple(
                                                    gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
                                                    2 * 60 * gst::SECOND).expect("Failed to seek");
                            did_cue = true;
                        }
                    }
                }
                _ => {
                    //println!("Message: {:?}", msg);
                },
            };

            glib::Continue(true)
        });

        main_loop.run();
}
