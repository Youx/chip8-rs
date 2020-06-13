// Note that a dynamic `import` statement here is required due to
// webpack/webpack#6615, but in theory `import { greet } from './pkg';`
// will work here one day as well!
const rust = import('./pkg');
const $ = require('jquery');

$(function() {
  var stop = true;
  var emu;

  rust.then((m) => {
    $("#reset").click(function() {
      stop = true;
    });
    $("#rom-file").change(function() {
      if (this.files) {
        this.files[0].arrayBuffer().then(function(data) {
          if (!emu) {
            emu = m.start(data);
            
            function register_key(i) {
              $("#key-" + i).mouseup(function() {
                emu.set_key_released(i);
              });
              $("#key-" + i).mousedown(function() {
                emu.set_key_pressed(i);
              });
            }
            console.log(emu);

            for (var i = 0; i < 16; i++) {
              register_key(i);
            }
          } else {
            stop = true;
            emu.reload(data);
          }
          stop = false;
          function render() {
            emu.run();
            if (!stop) {
              requestAnimationFrame(render);
            }
          }
          render();
        })
      }
    });
  })
  .catch(console.error);
});

