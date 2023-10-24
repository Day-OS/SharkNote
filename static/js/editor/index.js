import {DirVisualizer} from "../files.js"
import {dataURLtoFile, toBase64, toRAWB64} from '../utils.js'
const PAGE_SELECTED = new URLSearchParams(window.location.search).get('page_id');
const PARSER = new DOMParser();

const dir = new DirVisualizer(PAGE_SELECTED);

document.regex_dir_name = (element)=>{
    const button = element.form.querySelectorAll('button')[0];
    let reg = new RegExp("^[a-zA-Zа-яА-Я0-9_!]+$");
    let name_input_valid = reg.test(element.value);
    button.disabled = element.value == "" || !name_input_valid ;
    element.classList.toggle("is-invalid", !(element.value == "") && !name_input_valid)

}
document.regex_file_name = (element)=>{
    const button = element.form.querySelectorAll('button')[0];
    let reg = new RegExp("^[a-zA-Zа-яА-Я0-9](?:[a-zA-Zа-яА-Я0-9 ._-]*[a-zA-Z0-9])?\\.[a-zA-Z0-9_-]+$");
    let name_input_valid = reg.test(element.value);
    button.disabled = element.value == "" || !name_input_valid ;
    element.classList.toggle("is-invalid", !(element.value == "") && !name_input_valid)
}

/**
 * 
 * @param {String} form_id 
 */
document.file_upload_form_check = (form_id)=>{
    const form = document.getElementById(form_id)
    /**
     * @type {HTMLInputElement}
     */
    let input_file = form.querySelectorAll('input[type="file"]')[0]
    /**
     * @type {HTMLInputElement}
     */
    let input_name = form.querySelectorAll('input[type="text"]')[0]
    /**
     * @type {HTMLInputElement}
     */
    let input_submit = form.querySelectorAll('#create-file-button')[0]
    
    let name_empty = input_name.value == ""
    let file_not_loaded = input_file.value == ""
    input_name.disabled = !file_not_loaded
    input_file.disabled = !name_empty
    input_submit.disabled = name_empty && file_not_loaded 
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

    /*

    document.getElementById('file-modal').addEventListener('show.bs.modal', (evt) => {dir.update()})

    //FILE MODAL
    const file_modal = document.getElementById('file-creation-modal');
    const name_input_element = file_modal.querySelectorAll('#fileName')[0];
    const upload_element = file_modal.querySelectorAll('#fileUpload')[0];
    const reset = file_modal.querySelectorAll('#fileUploadRemove')[0];
    const create_button = file_modal.querySelectorAll('#create-file-button')[0];

    const update_form_file = () =>{
        let upload_element_empty = upload_element.value == "",
            name_input_element_empty = name_input_element.value == "";
        let reg = new RegExp("^[a-zA-Zа-яА-Я0-9](?:[a-zA-Zа-яА-Я0-9 ._-]*[a-zA-Z0-9])?\\.[a-zA-Z0-9_-]+$");
        let name_input_valid = reg.test(name_input_element.value);

        //Enabling and disabling buttons
        name_input_element.disabled = !upload_element_empty
        upload_element.disabled = !name_input_element_empty
        create_button.disabled = (name_input_element_empty || !name_input_valid ) && upload_element_empty

        // validating file name
        name_input_valid || name_input_element_empty ? name_input_element.classList.remove("is-invalid"): name_input_element.classList.add("is-invalid")
    }
    [upload_element, name_input_element, reset].forEach((e)=>{e.addEventListener("input", update_form_file ), e.addEventListener("click", ()=>{setTimeout(update_form_file,100)})})
    update_form_file()

    create_button.addEventListener("click", async ()=>{
        /**
         * @type {File[]}
         * /
        let files = []

        if (upload_element.disabled) {
            files.push(new File([""], name_input_element.value))
            
        }else{
            Array.from(upload_element.files).forEach((f)=>{files.push(f)})
        }
        for (let i = 0; i < files.length; i++) {
            const file = files[i];
            let formData = new FormData();
            formData.append("req_type", "file")
            formData.append("path", `${dir.pointer}/`)
            formData.append("content", file)
            

            let response = await fetch(`/editor/write/${PAGE_SELECTED}`, {
                method: 'POST',
                body:formData
            });
        }
    
    })
    //DIR MODAL


    const dir_modal = document.getElementById('dir-creation-modal');
    const dir_name_input_element = dir_modal.querySelectorAll('#dirName')[0];
    const dir_create_button = dir_modal.querySelectorAll('#create-dir-button')[0];

    

    const update_form_dir = () =>{
        let reg = new RegExp("^[a-zA-Zа-яА-Я0-9_!]+$");
        let input_empty = dir_name_input_element.value == "",
            input_valid = reg.test(dir_name_input_element.value);

        dir_create_button.disabled = input_empty || !input_valid 
        input_valid || input_empty ? dir_name_input_element.classList.remove("is-invalid"): dir_name_input_element.classList.add("is-invalid")
    }
    dir_name_input_element.addEventListener("input", update_form_dir );
    update_form_dir()

    dir_create_button.addEventListener("click", async ()=>{
        let formData = new FormData();
            formData.append("req_type", "dir")
            formData.append("path", `${dir.pointer}/${dir_name_input_element.value}`)
            let response = await fetch(`/editor/write/${PAGE_SELECTED}`, {
                method: 'POST',
                body:formData
            });
    
    })
    */

    //bootstrap.Modal.getOrCreateInstance(dir_modal).show();
    


    
}, false);
