extern crate imageflow_core;
extern crate libc;
extern crate rustc_serialize;
extern crate imageflow_types as s;
extern crate serde;
extern crate serde_json;

use std::ffi::CString;
use std::path::Path;

use imageflow_core::{Context, JsonResponse};

fn default_build_config(debug: bool) -> s::Build001Config {
    s::Build001Config{graph_recording: match debug{ true => Some(s::Build001GraphRecording::debug_defaults()), false => None} ,
        process_all_gif_frames: Some(false),
        enable_jpeg_block_scaling: Some(false),
        no_gamma_correction: false
    }
}


/// Creates a static, null-terminated Rust string, and
/// returns a ` *const libc::c_char` pointer to it.
///
/// Useful for API invocations that require a static C string

macro_rules! static_char {
    ($lit:expr) => {
        concat!($lit, "\0").as_ptr() as *const libc::c_char
    }
}

fn smoke_test(input: Option<s::IoEnum>, output: Option<s::IoEnum>,  debug: bool, steps: Vec<s::Node>){
    let mut io_list = Vec::new();
    if input.is_some() {
        io_list.push(s::IoObject {
            io_id: 0,
            direction: s::IoDirection::In,

            io: input.unwrap()
        });
    }
    if output.is_some() {
        io_list.push(s::IoObject {
            io_id: 1,
            direction: s::IoDirection::Out,

            io: output.unwrap()
        });
    }
    let build = s::Build001{
        builder_config: Some(default_build_config(debug)),
        io: io_list,
        framewise: s::Framewise::Steps(steps)
    };
    let mut context = Context::create().unwrap();
    context.message("v0.1/build", &serde_json::to_vec(&build).unwrap()).unwrap();
}

fn compare(input: Option<s::IoEnum>, allowed_off_by_one_bytes: usize, checksum_name: String, store_if_missing: bool, debug: bool, mut steps: Vec<s::Node>) -> bool {
    let mut dest_bitmap: *mut imageflow_core::ffi::BitmapBgra = std::ptr::null_mut();

    let ptr_to_ptr = &mut dest_bitmap as *mut *mut imageflow_core::ffi::BitmapBgra;

    let mut inputs = Vec::new();
    if input.is_some() {
        inputs.push(s::IoObject {
            io_id: 0,
            direction: s::IoDirection::In,

            io: input.unwrap()
        });
    }

    steps.push(s::Node::FlowBitmapBgraPtr { ptr_to_flow_bitmap_bgra_ptr: ptr_to_ptr as usize});

    {

        //println!("{}", serde_json::to_string_pretty(&steps).unwrap());
    }

    let build = s::Build001{
        builder_config: Some(s::Build001Config{graph_recording: match debug{ true => Some(s::Build001GraphRecording::debug_defaults()), false => None} ,
            process_all_gif_frames: Some(false),
            enable_jpeg_block_scaling: Some(false),
            no_gamma_correction: false
        }),
        io: inputs,
        framewise: s::Framewise::Steps(steps)
    };


    if debug {
        println!("{}", serde_json::to_string_pretty(&build).unwrap());
    }


    let mut context = Context::create().unwrap();

    context.message("v0.1/build", &serde_json::to_vec(&build).unwrap()).unwrap();

    unsafe {
        if debug {
            println!("{:?}", *ptr_to_ptr);
        }
    }

     unsafe {

         let matched: bool;
         let c_checksum_name = CString::new(checksum_name).unwrap();
         {
             let storage_relative_to = CString::new(Path::new(env!("CARGO_MANIFEST_DIR")).join(Path::new("tests")).join(Path::new("visuals.rs")).into_os_string().as_os_str().to_str().unwrap()).unwrap();
             let storage_rel =  storage_relative_to.as_bytes_with_nul().as_ptr();
             let storage_relative_to_ptr: *const i8 = ::std::mem::transmute(storage_rel);
             matched = imageflow_core::ffi::flow_bitmap_bgra_test_compare_to_record(context.flow_c(), *ptr_to_ptr, c_checksum_name.as_ptr(), store_if_missing, allowed_off_by_one_bytes, static_char!(file!()), 0, storage_relative_to_ptr);
         }
         context.error().assert_ok();

         return matched;
     }
}

#[test]
fn test_fill_rect(){
    let matched = compare(None, 500,
                          "FillRectEECCFF".to_owned(), false, false, vec![
        s::Node::CreateCanvas {w: 200, h: 200, format: s::PixelFormat::Bgra32, color: s::Color::Transparent},
        s::Node::FillRect{x1:0, y1:0, x2:100, y2:100, color: s::Color::Srgb(s::ColorSrgb::Hex("EECCFFFF".to_owned()))},
        s::Node::Resample2D{ w: 400, h: 400, down_filter: Some(s::Filter::Hermite), up_filter: Some(s::Filter::Hermite), hints: None }
        ]
    );
    assert!(matched);
}

#[test]
fn test_expand_rect(){
    let matched = compare(None, 500,
                          "FillRectEECCFFExpand2233AAFF".to_owned(), false, false, vec![
        s::Node::CreateCanvas {w: 200, h: 200, format: s::PixelFormat::Bgra32, color: s::Color::Transparent},
        s::Node::FillRect{x1:0, y1:0, x2:100, y2:100, color: s::Color::Srgb(s::ColorSrgb::Hex("EECCFFFF".to_owned()))},
        s::Node::ExpandCanvas{left: 10, top: 15, right: 20, bottom: 25, color: s::Color::Srgb(s::ColorSrgb::Hex("2233AAFF".to_owned()))},
        s::Node::Resample2D{ w: 400, h: 400, down_filter: Some(s::Filter::Hermite), up_filter: Some(s::Filter::Hermite), hints: None }
        ]
    );
    assert!(matched);
}


#[test]
fn test_crop(){
    let is_32bit = std::env::var("PLATFORM").and_then(|s| Ok(s.to_uppercase())) == Ok("X86".to_owned());
    let is_appveyor = std::env::var("APPVEYOR").and_then(|s| Ok(s.to_uppercase())) == Ok("TRUE".to_owned());

    if is_32bit && is_appveyor{
        println!("Skipping test_crop on Appveyor win32. Fails and I don't know why yet");
        return;
    }
    for _ in 1..100 {
        let matched = compare(None, 500,
                              "FillRectAndCrop".to_owned(), false, false, vec![
            s::Node::CreateCanvas { w: 200, h: 200, format: s::PixelFormat::Bgra32, color: s::Color::Srgb(s::ColorSrgb::Hex("FF5555FF".to_owned())) },
            s::Node::FillRect { x1: 0, y1: 0, x2: 10, y2: 100, color: s::Color::Srgb(s::ColorSrgb::Hex("0000FFFF".to_owned())) },
            s::Node::Crop { x1: 0, y1: 50, x2: 100, y2: 100 }
            ]
        );
        assert!(matched);
    }
}



//  Replaces TEST_CASE("Test scale rings", "")
#[test]
fn test_scale_rings(){
    let matched = compare(Some(s::IoEnum::Url("http://s3-us-west-2.amazonaws.com/imageflow-resources/test_inputs/rings2.png".to_owned())), 500,
        "RingsDownscaling".to_owned(), false, false, vec![
        s::Node::Decode {io_id: 0, commands: None},
        s::Node::Resample2D{ w: 400, h: 400, down_filter: Some(s::Filter::Hermite), up_filter: Some(s::Filter::Hermite), hints: None }
        ]
    );
    assert!(matched);
}

#[test]
fn test_fill_rect_original(){
    //let white = s::Color::Srgb(s::ColorSrgb::Hex("FFFFFFFF".to_owned()));
    let blue = s::Color::Srgb(s::ColorSrgb::Hex("0000FFFF".to_owned()));
    let matched = compare(None, 1,
                          "FillRect".to_owned(), false, false, vec![
        s::Node::CreateCanvas {w: 400, h: 300, format: s::PixelFormat::Bgra32, color: s::Color::Transparent},
        s::Node::FillRect{x1:0, y1:0, x2:50, y2:100, color: blue},
        ]
    );
    assert!(matched);
}

fn request_1d_twice_mode() -> s::ResampleHints {
    s::ResampleHints {
        sharpen_percent: None,
        prefer_1d_twice: Some(true)
    }
}

#[test]
fn test_scale_image() {
    let matched = compare(Some(s::IoEnum::Url("http://s3-us-west-2.amazonaws.com/imageflow-resources/test_inputs/waterhouse.jpg".to_owned())), 500,
                          "ScaleTheHouse".to_owned(), false, false, vec![
        s::Node::Decode {io_id: 0, commands: None},
        s::Node::Resample2D{ w: 400, h: 300, down_filter: Some(s::Filter::Robidoux), up_filter: Some(s::Filter::Robidoux), hints: Some(request_1d_twice_mode()) }
        ]
    );
    assert!(matched);
}



#[test]
fn test_jpeg_icc2_color_profile() {
    let matched = compare(Some(s::IoEnum::Url("http://s3-us-west-2.amazonaws.com/imageflow-resources/test_inputs/MarsRGB_tagged.jpg".to_owned())), 500,
                          "MarsRGB_ICC_Scaled400300".to_owned(), false, false, vec![
s::Node::Decode {io_id: 0, commands: None},
s::Node::Resample2D{ w: 400, h: 300, down_filter: Some(s::Filter::Robidoux), up_filter: Some(s::Filter::Robidoux), hints: Some(request_1d_twice_mode()) }
]
    );
    assert!(matched);
}

#[test]
fn test_jpeg_icc4_color_profile() {
    let matched = compare(Some(s::IoEnum::Url("http://s3-us-west-2.amazonaws.com/imageflow-resources/test_inputs/MarsRGB_v4_sYCC_8bit.jpg".to_owned())), 500,
                          "MarsRGB_ICCv4_Scaled400300".to_owned(), false, false, vec![
s::Node::Decode {io_id: 0, commands: None},
s::Node::Resample2D{ w: 400, h: 300, down_filter: Some(s::Filter::Robidoux), up_filter: Some(s::Filter::Robidoux), hints: Some(request_1d_twice_mode()) }
]
    );
    assert!(matched);
}

#[test]
fn test_jpeg_rotation() {
    let orientations = vec!["Landscape", "Portrait"];

    for orientation in orientations {
        for flag in 1..9 {
            let url = format!("http://s3-us-west-2.amazonaws.com/imageflow-resources/test_inputs/orientation/{}_{}.jpg", orientation, flag);
            let title = format!("Test_Apply_Orientation_{}_{}.jpg", orientation, flag);
            let matched = compare(Some(s::IoEnum::Url(url)), 500, title, false, false, vec![s::Node::Decode {io_id: 0, commands: None}]);
            assert!(matched);
        }
    }

}


#[test]
fn test_encode_jpeg_smoke() {
    let steps = vec![
    s::Node::Decode {io_id: 0, commands: None},
    s::Node::Resample2D{ w: 400, h: 300, down_filter: Some(s::Filter::Robidoux), up_filter: Some(s::Filter::Robidoux), hints: None },
    s::Node::Encode{ io_id: 1, preset: s::EncoderPreset::LibjpegTurbo {quality: Some(100)}}
    ];

    smoke_test(Some(s::IoEnum::Url("http://s3-us-west-2.amazonaws.com/imageflow-resources/test_inputs/MarsRGB_v4_sYCC_8bit.jpg".to_owned())),
               Some(s::IoEnum::OutputBuffer),
               false,
               steps,
    );
}

#[test]
fn test_encode_png32_smoke() {
    let steps = vec![
    s::Node::Decode {io_id: 0, commands: None},
    s::Node::Resample2D{ w: 400, h: 300, down_filter: Some(s::Filter::Robidoux), up_filter: Some(s::Filter::Robidoux), hints: None },
    s::Node::FlipV,
    s::Node::Crop{ x1: 20, y1: 20, x2: 380, y2: 280},
    s::Node::Encode{ io_id: 1, preset: s::EncoderPreset::Libpng {depth: Some(s::PngBitDepth::Png32), matte: None,  zlib_compression: None}}
    ];

    smoke_test(Some(s::IoEnum::Url("http://s3-us-west-2.amazonaws.com/imageflow-resources/test_inputs/MarsRGB_v4_sYCC_8bit.jpg".to_owned())),
               Some(s::IoEnum::OutputBuffer),
               false,
               steps,
    );
}

fn get_result_dimensions(steps: Vec<s::Node>, io: Vec<s::IoObject>, debug: bool) -> (u32, u32) {
    let mut steps = steps.clone();

    let mut dest_bitmap: *mut imageflow_core::ffi::BitmapBgra = std::ptr::null_mut();
    let ptr_to_ptr = &mut dest_bitmap as *mut *mut imageflow_core::ffi::BitmapBgra;

    steps.push(s::Node::FlowBitmapBgraPtr { ptr_to_flow_bitmap_bgra_ptr: ptr_to_ptr as usize});

    let build = s::Build001{
        builder_config: Some(default_build_config(debug)),
        io: io,
        framewise: s::Framewise::Steps(steps)
    };
    let mut context = Context::create().unwrap();
    context.message("v0.1/build", &serde_json::to_vec(&build).unwrap()).unwrap();
    unsafe { ((*dest_bitmap).w, (*dest_bitmap).h) }
}


#[test]
fn test_dimensions(){
    let steps = vec![
    s::Node::CreateCanvas{w: 638, h: 423, format: s::PixelFormat::Bgra32, color: s::Color::Black},
    //s::Node::Crop { x1: 0, y1: 0, x2: 638, y2: 423},
    s::Node::Resample2D{w:200,h:133, down_filter: None, up_filter: None, hints: None},
    s::Node::ExpandCanvas{left:1, top: 0, right:0, bottom: 0, color: s::Color::Transparent},
    ];
    let (w, h) = get_result_dimensions(steps, vec![], false);
    assert_eq!(w,201);
    assert_eq!(h,133);

}




#[test]
fn test_decode_png_and_scale_dimensions(){

    let tinypng = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, 0x00,
    0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00,
    0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01,
        0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82 ];

    let png = s::IoObject{
        io_id: 0,
        direction: s::IoDirection::In,

        io: s::IoEnum::ByteArray(tinypng)
    };
    let steps = vec![
    s::Node::Decode{io_id: 0, commands: None},
    //s::Node::Crop { x1: 0, y1: 0, x2: 638, y2: 423},
    s::Node::Resample2D{w:300,h:200,  down_filter: None, up_filter: None, hints: None},
    ];
    let (w, h) = get_result_dimensions(steps, vec![png], false);
    assert_eq!(w,300);
    assert_eq!(h,200);

}


fn test_idct_callback(_: s::ImageInfo) -> (Option<s::DecoderCommand>, Vec<s::Node>, bool)
{
    let new_w = (800 * 4 + 8 - 1) / 8;
    let new_h = (600 * 4 + 8 - 1) / 8;
    let hints = s::JpegIDCTDownscaleHints{
        gamma_correct_for_srgb_during_spatial_luma_scaling: Some(true),
        scale_luma_spatially: Some(true),
        width: new_w,
        height: new_h
    };
    (Some(s::DecoderCommand::JpegDownscaleHints(hints)), vec![s::Node::Decode{io_id:0, commands: None}], false)

}
//fn test_idct_callback_no_gamma(_: s::ImageInfo) -> (Option<s::TellDecoderWhat>, Vec<s::Node>, bool)
//{
//    let new_w = (800 * 4 + 8 - 1) / 8;
//    let new_h = (600 * 4 + 8 - 1) / 8;
//    let hints = s::JpegIDCTDownscaleHints{
//        gamma_correct_for_srgb_during_spatial_luma_scaling: Some(false),
//        scale_luma_spatially: Some(true),
//        width: new_w,
//        height: new_h
//    };
//    (Some(s::TellDecoderWhat::JpegDownscaleHints(hints)), vec![s::Node::Decode{io_id:0}], false)
//
//}
//

fn test_idct_no_gamma_callback(info: s::ImageInfo) -> (Option<s::DecoderCommand>, Vec<s::Node>, bool)
{
    let new_w = (info.image_width * 6 + 8 - 1) / 8;
    let new_h = (info.image_height * 6 + 8 - 1) / 8;
    let hints = s::JpegIDCTDownscaleHints{
        gamma_correct_for_srgb_during_spatial_luma_scaling: Some(false),
        scale_luma_spatially: Some(true),
        width: new_w as i64,
        height: new_h as i64
    };
    //Here we send the hints via the Decode node instead.
    (None, vec![s::Node::Decode{io_id:0, commands: Some(vec![s::DecoderCommand::JpegDownscaleHints(hints)])}], false)

}

#[test]
fn test_idct_linear(){
    let matched = test_with_callback("ScaleIDCTFastvsSlow".to_owned(), s::IoEnum::Url("http://s3-us-west-2.amazonaws.com/imageflow-resources/test_inputs/roof_test_800x600.jpg".to_owned()),
    test_idct_callback);
    assert!(matched);
}

#[test]
fn test_idct_spatial_no_gamma(){
    let matched = test_with_callback("ScaleIDCT_approx_gamma".to_owned(), s::IoEnum::Url("http://s3.amazonaws.com/resizer-images/u1.jpg".to_owned()),
                                     test_idct_no_gamma_callback);
    assert!(matched);
}
//
//#[test]
//fn test_fail(){
//    let matched = test_with_callback("ScaleIDCTFastvsSlow".to_owned(), s::IoEnum::Url("http://s3-us-west-2.amazonaws.com/imageflow-resources/test_inputs/roof_test_800x600.jpg".to_owned()),
//                                     test_idct_callback_no_gamma);
//    assert!(matched);
//}

fn test_with_callback(checksum_name: String, input: s::IoEnum, callback: fn(s::ImageInfo) -> (Option<s::DecoderCommand>, Vec<s::Node>, bool) ) -> bool{
    let context = Context::create().unwrap();
    let matched:bool;

    unsafe {
        let mut job = context.create_job();
        //Add input
        ::imageflow_core::parsing::IoTranslator::new(&context).add_to_job(&mut *job, vec![s::IoObject{ io_id:0, direction: s::IoDirection::In, io: input}]);


        let info_blob: JsonResponse = job.message("v0.1/get_image_info", "{\"io_id\": 0}".as_bytes()).unwrap();
        let info_response: s::Response001 = serde_json::from_slice(info_blob.response_json.as_ref()).unwrap();
        if !info_response.success {
            panic!("get_image_info failed: {:?}",info_response);
        }
        let image_info = match info_response.data {
            s::ResponsePayload::ImageInfo(info) => info,
            _ => panic!("")
        };

        let (tell_decoder, mut steps, no_gamma_correction): (Option<s::DecoderCommand>, Vec<s::Node>,bool) = callback(image_info);

        if let Some(what) = tell_decoder {
            let send_hints = s::TellDecoder001 {
                io_id: 0,
                command: what
            };
            let send_hints_str = serde_json::to_string_pretty(&send_hints).unwrap();
            job.message("v0.1/tell_decoder", send_hints_str.as_bytes()).unwrap().assert_ok();
        }

        let mut dest_bitmap: *mut imageflow_core::ffi::BitmapBgra = std::ptr::null_mut();

        let ptr_to_ptr = &mut dest_bitmap as *mut *mut imageflow_core::ffi::BitmapBgra;

        steps.push(s::Node::FlowBitmapBgraPtr { ptr_to_flow_bitmap_bgra_ptr: ptr_to_ptr as usize});


        let send_execute = s::Execute001{
            framewise: s::Framewise::Steps(steps),
            graph_recording: None,
            no_gamma_correction: Some(no_gamma_correction)
        };

        let send_execute_str = serde_json::to_string_pretty(&send_execute).unwrap();
        job.message("v0.1/execute", send_execute_str.as_bytes()).unwrap().assert_ok();



            let store_if_missing = false;
            let allowed_off_by_one_bytes = 500;

            let c_checksum_name = CString::new(checksum_name).unwrap();
            {
                let storage_relative_to = CString::new(Path::new(env!("CARGO_MANIFEST_DIR")).join(Path::new("tests")).join(Path::new("visuals.rs")).into_os_string().as_os_str().to_str().unwrap()).unwrap();
                let storage_rel =  storage_relative_to.as_bytes_with_nul().as_ptr();
                let storage_relative_to_ptr: *const i8 = ::std::mem::transmute(storage_rel);
                matched = imageflow_core::ffi::flow_bitmap_bgra_test_compare_to_record(context.flow_c(), *ptr_to_ptr, c_checksum_name.as_ptr(), store_if_missing, allowed_off_by_one_bytes, static_char!(file!()), 0, storage_relative_to_ptr);
            }

            context.error().assert_ok();




    }
    context.destroy_allowing_panics();
    matched
}

//TODO: Consider adding test for flow_bitmap_bgra_sharpen_block_edges if we ever bring it back