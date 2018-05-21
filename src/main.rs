#![recursion_limit = "2048"]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate stdweb;

use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use stdweb::web::event::{IEvent, IKeyboardEvent, KeyDownEvent, KeyUpEvent, KeyboardLocation};
use stdweb::web::{self, Element, IElement, IEventTarget, INode, INonElementParentNode};

use stdweb::{UnsafeTypedArray, Value};

mod common;
use common::*;

mod game;
use game::update_and_render;

macro_rules! enclose {
    ( [$( $x:ident ),*] $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

// This creates a really basic WebGL context for blitting a single texture.
// On some web browsers this is faster than using a 2d canvas.
fn setup_webgl(canvas: &Element) -> Value {
    const FRAGMENT_SHADER: &'static str = r#"
        precision mediump float;
        varying vec2 v_texcoord;
        uniform sampler2D u_sampler;
        void main() {
            gl_FragColor = vec4( texture2D(
                u_sampler,
                vec2( v_texcoord.s, v_texcoord.t ) ).rgb,
                1.0
             );
        }
    "#;

    const VERTEX_SHADER: &'static str = r#"
        attribute vec2 a_position;
        attribute vec2 a_texcoord;
        uniform mat4 u_matrix;
        varying vec2 v_texcoord;
        void main() {
            gl_Position = u_matrix * vec4( a_position, 0.0, 1.0 );
            v_texcoord = a_texcoord;
        }
    "#;

    fn ortho(left: f64, right: f64, bottom: f64, top: f64) -> Vec<f64> {
        let mut m = vec![
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];

        m[0 * 4 + 0] = 2.0 / (right - left);
        m[1 * 4 + 1] = 2.0 / (top - bottom);
        m[3 * 4 + 0] = (right + left) / (right - left) * -1.0;
        m[3 * 4 + 1] = (top + bottom) / (top - bottom) * -1.0;

        return m;
    }

    js!(
        var gl;
        var webgl_names = ["webgl", "experimental-webgl", "webkit-3d", "moz-webgl"];
        for( var i = 0; i < webgl_names.length; ++i ) {
            var name = webgl_names[ i ];
            try {
                gl = @{canvas}.getContext( name );
            } catch( err ) {}

            if( gl ) {
                console.log( "WebGL support using context:", name );
                break;
            }
        }

        if( gl === null ) {
            console.error( "WebGL rendering context not found." );
            return null;
        }

        var vertex_shader = gl.createShader( gl.VERTEX_SHADER );
        var fragment_shader = gl.createShader( gl.FRAGMENT_SHADER );
        gl.shaderSource( vertex_shader, @{VERTEX_SHADER} );
        gl.shaderSource( fragment_shader, @{FRAGMENT_SHADER} );
        gl.compileShader( vertex_shader );
        gl.compileShader( fragment_shader );

        if( !gl.getShaderParameter( vertex_shader, gl.COMPILE_STATUS ) ) {
            console.error( "WebGL vertex shader compilation failed:", gl.getShaderInfoLog( vertex_shader ) );
            return null;
        }

        if( !gl.getShaderParameter( fragment_shader, gl.COMPILE_STATUS ) ) {
            console.error( "WebGL fragment shader compilation failed:", gl.getShaderInfoLog( fragment_shader ) );
            return null;
        }

        var program = gl.createProgram();
        gl.attachShader( program, vertex_shader );
        gl.attachShader( program, fragment_shader );
        gl.linkProgram( program );
        if( !gl.getProgramParameter( program, gl.LINK_STATUS ) ) {
            console.error( "WebGL program linking failed!" );
            return null;
        }

        gl.useProgram( program );

        var vertex_attr = gl.getAttribLocation( program, "a_position" );
        var texcoord_attr = gl.getAttribLocation( program, "a_texcoord" );

        gl.enableVertexAttribArray( vertex_attr );
        gl.enableVertexAttribArray( texcoord_attr );

        var sampler_uniform = gl.getUniformLocation( program, "u_sampler" );
        gl.uniform1i( sampler_uniform, 0 );

        var matrix = @{ortho( 0.0, 128.0, 128.0, 0.0 )};
        var matrix_uniform = gl.getUniformLocation( program, "u_matrix" );
        gl.uniformMatrix4fv( matrix_uniform, false, matrix );

        var texture = gl.createTexture();
        gl.bindTexture( gl.TEXTURE_2D, texture );
        gl.texImage2D(
            gl.TEXTURE_2D,
            0,
            gl.RGBA,
            128,
            128,
            0,
            gl.RGBA,
            gl.UNSIGNED_BYTE,
            new Uint8Array( 128 * 128 * 4 )
          );
        gl.texParameteri( gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST );
        gl.texParameteri( gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST );

        var vertex_buffer = gl.createBuffer();
        gl.bindBuffer( gl.ARRAY_BUFFER, vertex_buffer );
        var vertices = [
            0.0, 0.0,
            0.0, 128.0,
            128.0, 0.0,
            128.0, 128.0
        ];
        gl.bufferData( gl.ARRAY_BUFFER, new Float32Array( vertices ), gl.STATIC_DRAW );
        gl.vertexAttribPointer( vertex_attr, 2, gl.FLOAT, false, 0, 0 );

        var texcoord_buffer = gl.createBuffer();
        gl.bindBuffer( gl.ARRAY_BUFFER, texcoord_buffer );
        var texcoords = [
            0.0, 0.0,
            0.0, 128.0 / 128.0,
            1.0, 0.0,
            1.0, 128.0 / 128.0
        ];
        gl.bufferData( gl.ARRAY_BUFFER, new Float32Array( texcoords ), gl.STATIC_DRAW );
        gl.vertexAttribPointer( texcoord_attr, 2, gl.FLOAT, false, 0, 0 );

        var index_buffer = gl.createBuffer();
        gl.bindBuffer( gl.ELEMENT_ARRAY_BUFFER, index_buffer );
        var indices = [
            0, 1, 2,
            2, 3, 1
        ];
        gl.bufferData( gl.ELEMENT_ARRAY_BUFFER, new Uint16Array( indices ), gl.STATIC_DRAW );

        gl.clearColor( 0.0, 0.0, 0.0, 1.0 );
        gl.enable( gl.DEPTH_TEST );
        gl.viewport( 0, 0, 128, 128 );

        return gl;
    )
}

struct PinkyWeb {
    state: State,
    paused: bool,
    busy: bool,
    js_ctx: Value,
}

impl PinkyWeb {
    fn new(canvas: &Element) -> Self {
        let gl = setup_webgl(&canvas);

        let js_ctx = js!(
            var h = {};
            var canvas = @{canvas};

            h.gl = @{gl};

            if( !h.gl ) {
                console.log( "No WebGL; using Canvas API" );

                // If the WebGL **is** supported but something else
                // went wrong the web browser won't let us create
                // a normal canvas context on a WebGL-ified canvas,
                // so we recreate a new canvas here to work around that.
                var new_canvas = canvas.cloneNode( true );
                canvas.parentNode.replaceChild( new_canvas, canvas );
                canvas = new_canvas;

                h.ctx = canvas.getContext( "2d" );
                h.img = h.ctx.createImageData( 128, 128 );
                h.buffer = new Uint32Array( h.img.data.buffer );
            }

            return h;
        );

        PinkyWeb {
            state: State::new(),
            paused: true,
            busy: false,
            js_ctx,
        }
    }

    fn pause(&mut self) {
        self.paused = true;
    }

    fn unpause(&mut self) {
        self.paused = false;
        self.busy = false;
    }

    fn execute_cycle(&mut self) -> Result<bool, Box<Error>> {
        self.state.frame();

        Ok(true)
    }

    fn run_a_bit(&mut self) -> Result<bool, Box<Error>> {
        if self.paused {
            return Ok(true);
        }

        loop {
            let result = self.execute_cycle();
            match result {
                Ok(processed_whole_frame) => {
                    if processed_whole_frame {
                        return Ok(true);
                    }
                }
                Err(error) => {
                    js!( console.error( "Execution error:", @{format!( "{}", error )} ); );
                    self.pause();

                    return Err(error);
                }
            }
        }
    }

    fn draw(&mut self) {
        if !self.paused {
            js! {
                var h = @{&self.js_ctx};
                var framebuffer = @{unsafe {
                    UnsafeTypedArray::new( &self.state.framebuffer.buffer )
                 }};
                if( h.gl ) {
                    var data = new Uint8Array(
                        framebuffer.buffer,
                        framebuffer.byteOffset,
                        framebuffer.byteLength
                    );
                    h.gl.texSubImage2D( h.gl.TEXTURE_2D,
                         0, 0, 0, 128, 128, h.gl.RGBA, h.gl.UNSIGNED_BYTE, data );
                    h.gl.drawElements( h.gl.TRIANGLES, 6, h.gl.UNSIGNED_SHORT, 0 );
                } else {
                    h.buffer.set( framebuffer );
                    h.ctx.putImageData( h.img, 0, 0 );
                }
            }
        }
    }

    fn on_key(&mut self, key: &str, location: KeyboardLocation, is_pressed: bool) -> bool {
        let button = match (key, location) {
            ("Enter", _) => Button::Start,
            ("Shift", KeyboardLocation::Right) => Button::Select,
            ("ArrowUp", _) => Button::Up,
            ("ArrowLeft", _) => Button::Left,
            ("ArrowRight", _) => Button::Right,
            ("ArrowDown", _) => Button::Down,

            // On Edge the arrows have different names
            // for some reason.
            ("Up", _) => Button::Up,
            ("Left", _) => Button::Left,
            ("Right", _) => Button::Right,
            ("Down", _) => Button::Down,

            ("z", _) => Button::A,
            ("x", _) => Button::B,

            // For those using the Dvorak layout.
            (";", _) => Button::A,
            ("q", _) => Button::B,

            // For those using the Dvorak layout **and** Microsoft Edge.
            //
            // On `keydown` we get ";" as we should, but on `keyup`
            // we get "Unidentified". Seriously Microsoft, how buggy can
            // your browser be?
            ("Unidentified", _) if is_pressed == false => Button::A,

            _ => return false,
        };

        PinkyWeb::set_button_state(self, button, is_pressed);
        return true;
    }

    fn set_button_state(&mut self, button: Button::Ty, is_pressed: bool) {
        if is_pressed {
            self.state.press(button);
        } else {
            self.state.release(button);
        }
    }
}

impl State {
    pub fn frame(&mut self) {
        update_and_render(&mut self.framebuffer, &mut self.game_state, self.input);

        self.input.previous_gamepad = self.input.gamepad;
    }

    pub fn press(&mut self, button: Button::Ty) {
        self.input.gamepad.insert(button);
    }

    pub fn release(&mut self, button: Button::Ty) {
        self.input.gamepad.remove(button);
    }
}

fn emulate_for_a_single_frame(pinky: Rc<RefCell<PinkyWeb>>) {
    pinky.borrow_mut().busy = true;

    web::set_timeout(
        enclose!( [pinky] move || {
        let finished_frame = match pinky.borrow_mut().run_a_bit() {
            Ok( result ) => result,
            Err( error ) => {
                handle_error( error );
                return;
            }
        };

        if !finished_frame {
            web::set_timeout( move || { emulate_for_a_single_frame( pinky ); }, 0 );
        } else {
            let mut pinky = pinky.borrow_mut();
            pinky.busy = false;
        }
    }),
        0,
    );
}

fn main_loop(pinky: Rc<RefCell<PinkyWeb>>) {
    // If we're running too slowly there is no point
    // in queueing up even more work.
    if !pinky.borrow_mut().busy {
        emulate_for_a_single_frame(pinky.clone());
    }

    pinky.borrow_mut().draw();
    web::window().request_animation_frame(move |_| {
        main_loop(pinky);
    });
}

fn show(id: &str) {
    web::document()
        .get_element_by_id(id)
        .unwrap()
        .class_list()
        .remove("hidden")
        .unwrap();
}

fn hide(id: &str) {
    web::document()
        .get_element_by_id(id)
        .unwrap()
        .class_list()
        .add("hidden")
        .unwrap();
}

fn support_input(pinky: Rc<RefCell<PinkyWeb>>) {
    web::window().add_event_listener(enclose!( [pinky] move |event: KeyDownEvent| {
        let handled = pinky.borrow_mut().on_key( &event.key(), event.location(), true );
        if handled {
            event.prevent_default();
        }
    }));

    web::window().add_event_listener(enclose!( [pinky] move |event: KeyUpEvent| {
        let handled = pinky.borrow_mut().on_key( &event.key(), event.location(), false );
        if handled {
            event.prevent_default();
        }
    }));
}

fn handle_error<E: Into<Box<Error>>>(error: E) {
    let error_message = format!("{}", error.into());
    web::document()
        .get_element_by_id("error-description")
        .unwrap()
        .set_text_content(&error_message);

    hide("viewport");
    show("error");
}

fn main() {
    stdweb::initialize();

    let canvas = web::document().get_element_by_id("viewport").unwrap();
    let pinky = Rc::new(RefCell::new(PinkyWeb::new(&canvas)));

    support_input(pinky.clone());

    hide("loading");
    hide("error");

    pinky.borrow_mut().unpause();

    show("viewport");

    web::window().request_animation_frame(move |_| {
        main_loop(pinky);
    });

    stdweb::event_loop();
}
