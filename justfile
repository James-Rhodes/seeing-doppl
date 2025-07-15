crate_name := `echo ${PWD##*/}`
index_file := "<html lang='en'>

<head>
    <meta charset='utf-8'>
    <title>" + crate_name + "</title>
    <style>
        html,
        body,
        canvas {
            margin: 0px;
            padding: 0px;
            width: 100%;
            height: 100%;
            overflow: hidden;
            position: absolute;
            background: black;
            z-index: 0;
        }
    </style>
</head>

<body>
    <canvas id='" + crate_name + "' tabindex='1'></canvas>
    <script src='./mq_js_bundle.js'></script>
    <script>load('" + crate_name + ".wasm');</script> 
</body>

</html>"

# Build the current project for the web along with template files
build-web: 
    @# build the program
    cargo build --profile=web-release --target wasm32-unknown-unknown

    @# get the wasm js files from the macroquad repo
    wget -O ./target/wasm32-unknown-unknown/web-release/mq_js_bundle.js https://raw.githubusercontent.com/not-fl3/macroquad/master/js/mq_js_bundle.js 
    
    @# add this crate name as the canvas id
    @sed -i -e 's/#glcanvas/#seeing_doppl/g' ./target/wasm32-unknown-unknown/web-release/mq_js_bundle.js
    @sed -i -e 's/canvas.focus(),//g' ./target/wasm32-unknown-unknown/web-release/mq_js_bundle.js
    @echo "export{load};" >> ./target/wasm32-unknown-unknown/web-release/mq_js_bundle.js
    @echo "" > ./target/wasm32-unknown-unknown/web-release/index.html
    
    @# run wasm binary optimization
    wasm-opt -Oz -o ./target/wasm32-unknown-unknown/web-release/seeing_doppl.wasm ./target/wasm32-unknown-unknown/web-release/seeing_doppl.wasm 
    wasm-snip ./target/wasm32-unknown-unknown/web-release/seeing_doppl.wasm  -o ./target/wasm32-unknown-unknown/web-release/seeing_doppl.wasm 

