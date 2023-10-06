import {DirVisualizer} from "./files.js"
const PAGE_SELECTED = new URLSearchParams(window.location.search).get('page_id');
const PARSER = new DOMParser();

const dir = new DirVisualizer(PAGE_SELECTED);

/**
 * Get files content and sends it to the editor
 * @param {String} path 
 */
async function open_file(path){
    let response = await fetch(`${PAGE_SELECTED}${path}`, {
        method: 'GET',
        headers: {
        },
    });
    bootstrap.Modal.getInstance('#file-modal').hide();
    ace.edit("editor").setValue(await response.text());
}

/**
 * Sends confirmation Modal
 * @param {String} name 
 * @param {String} path 
 */
function delete_from_path(name, path){
    let modal = $("#delete-modal")
    modal.find(".file-name").html(name)
    modal.find(".delete").click(()=>{
        delete_from_path_conf(path)
        modal.modal("hide")
    })
    modal.modal("show");
}
/**
 * Sends confirmation Modal
 * @param {String} path 
 */
function delete_from_path_conf(path){
    alert(`apagoukk ${path}`)
}

document.addEventListener('DOMContentLoaded', () =>{
    //https://ace.c9.io/#nav=howto
    const editor = ace.edit("editor");
    editor.commands.addCommand({
        name: 'save',
        bindKey: { win: "Ctrl-S", "mac": "Cmd-S" },
        exec: function (editor) {
            console.log("saving", editor.session.getValue());
        }
    });
    editor.resize();
    editor.setTheme("ace/theme/twilight");
    editor.session.setMode("ace/mode/markdown");

    document.getElementById('file-modal').addEventListener('show.bs.modal', (evt) => {dir.update()})

    
}, false);
