const PAGE_SELECTED = new URLSearchParams(window.location.search).get('page');
function select_page(element){
    window.location.href = "/editor?page=" + element.value;
}

function load_file(element){
    var request = new XMLHttpRequest()
    request.open( "GET", "/content/"+ PAGE_SELECTED +"/" + element.value , false ); // false for synchronous request
    request.setRequestHeader("Content-Type", "application/x-www-form-urlencoded");
    request.addEventListener("load", ()=> {
        // if the request succeeded
        if (request.status >= 200 && request.status < 300) {
            // print the response to the request
            console.log(request.response);
        }

    }) 
    request.send( "username=Dani%40asd.c&password=d" );
}

window.onload = ()=>{
    Split(['#split-0', '#split-1', '#split-2'],{ 
        sizes: [20, 80, 0],
        minSize: [30, 30, 0],
    })
        //https://ace.c9.io/#nav=howto
    var editor = ace.edit("editor");
    editor.resize();
    var md = ace.createEditSession("some js code");
    //console.log(editor.getValue())
    editor.setTheme("ace/theme/twilight");
    editor.session.setMode("ace/mode/markdown");
}