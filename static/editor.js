const PAGE_SELECTED = new URLSearchParams(window.location.search).get('page');

class File {
    constructor(name, content, saved, time_saved){

    }
}

function select_page(element){
    window.location.href = "/editor?page=" + element.value;
}

function load_file(element){
    element.dataset.filelocation
    var request = new XMLHttpRequest()
    request.open( "GET", "/content/"+ element.dataset.filelocation , false ); // false for synchronous request
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
    //https://ace.c9.io/#nav=howto
    var editor = ace.edit(document.getElementById("editor"));
    editor.commands.addCommand({
        name: 'save',
        bindKey: {win: "Ctrl-S", "mac": "Cmd-S"},
        exec: function(editor) {
            console.log("saving", editor.session.getValue())
        }
    })
    editor.resize();
    var md = ace.createEditSession("some js code");
    //console.log(editor.getValue())
    editor.setTheme("ace/theme/twilight");
    editor.session.setMode("ace/mode/markdown");
}