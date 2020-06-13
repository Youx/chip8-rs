// all pixels are white
pub const SHADER: &str = r#"
    precision mediump float;

    void main() {
        gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
    }
"#;