export class DirVisualizer {
    /**
     * @param {Directory} root 
     * @param {String} pointer 
     */
    constructor(PAGE_SELECTED, root = new Directory(""), pointer = ""){
        this.root = root;
        this.pointer = pointer;
        this.PAGE_SELECTED = PAGE_SELECTED;
    }
    /**
     * @returns {String}
    */
    getDirName() {
        this.pointer.split("/").slice(-1)
    }

    /**
     * This will be like: root/a/b
     * @param {String} path 
     * @returns {Directory}
     */
    getDir(path){
        const Path = path.split("/")
        let current_dir = this.root
        for (let i = 0; i < Path.length; i++) {
            current_dir.directories.forEach(directory => {
                if (directory.name == Path[i]) {
                    current_dir = directory
                 }
            });
        }
        return current_dir;
    }
    /**
     * This will be like: root/a/b
     * @param {String} path 
     * @returns {Directory | null}
     */
    getParentPath(path){
        let Path = path.split("/")
        Path.pop()
        if (Path.length < 0) {
            return null
        }
        else if (Path.length == 1) {
            return "/"
        }
        return Path.join("/")
        
    }
    async update(){
        console.log(this.pointer);
        let response = await fetch(`dir/${this.PAGE_SELECTED}`, {
            method: 'GET',
            headers: {
              'Content-Type': 'application/json'
            },
        });
        this.root = await response.json()

        console.log(response);
        console.log(this.root);
          
        let directory = this.getDir(this.pointer)
        let grid = document.getElementById('file-grid').content.firstElementChild.cloneNode(true);
        
        //RETURN BUTTON
        let returncard = document.getElementById('return').content.firstElementChild.cloneNode(true);
        let go_back_path = this.getParentPath(this.pointer)
        if (go_back_path) {
            returncard.querySelector("a").addEventListener("click", ()=>{
                let Path = this.pointer.split("/")
                Path.pop()
                this.pointer = Path.join("/")
                this.update()
            })
            grid.append(returncard)
        }

        ///DIRECTORIES BUTTONS
        for (let i = 0; i < directory.directories.length; i++) {
            const dir = directory.directories[i]
            let dircard = document.getElementById('directory').content.firstElementChild.cloneNode(true);
            dircard.querySelector(".card-title").innerHTML = dir.name;
            dircard.querySelector(".icon").addEventListener('click', ()=>{
                this.pointer = `${this.pointer}/${dir.name}`
                this.update()
            });
            dircard.querySelector(".rename").addEventListener("click", ()=>{rename(this.pointer + "/" + directory.name)})
            dircard.querySelector(".delete").addEventListener("click", ()=>{delete_from_path(dir.name, this.pointer + "/" + dir.name)})
            grid.append(dircard)
        }
        ///FILES BUTTONS
        for (let i = 0; i < directory.files.length; i++) {
            const file = directory.files[i];
            let ext_arr = file.name.split(".")
            const icon = ext_arr.length > 1? `bi-filetype-${ext_arr.slice(-1)}`: "bi-file-earmark-minus"
            
            
            let filecard = document.getElementById('file').content.firstElementChild.cloneNode(true);
            filecard.querySelector(".card-title").innerHTML = file.name
            filecard.querySelector(".card-img-top").classList.add(icon)
            filecard.querySelectorAll(".edit, .icon").forEach((e)=>{
                e.addEventListener("click", ()=>{open_file(this.pointer + "/" + file.name)})
            })
            filecard.querySelector(".rename").addEventListener("click", ()=>{rename(this.pointer + "/" + file.name)})
            filecard.querySelector(".delete").addEventListener("click", ()=>{delete_from_path(file.name, this.pointer + "/" + file.name)})
            grid.append(filecard)
        }
    
        let body = document.getElementById('file-modal').querySelector(".modal-body");
        body.innerHTML = "";
        body.append(grid);
    }
}
export class Directory {
    /**
     * @param {String} name 
     * @param {File[]} files 
     * @param {Directory[]} directories
     */
    constructor(name, files = [], directories = []) {
        this.name = name;
        this.files = files  //File
        this.directories = directories;
    }
}
export class File {
    /**
     * @param {String} name 
     */
    constructor(name) {
        this.name = name;
    }
}

export class CreationReq{
    constructor(dir_path, new_name){
        this.dir_path = path;
        this.new_name = new_name;
    }
}
/**
 * Used to delete send the POST Request for file re-nomination
 * Insert the path to the in it. A file or a directory
 */
export class RenominationReq{
    constructor(path, new_name){
        this.path = path;
        this.new_name = new_name;
    }
}
/**
 * Used to delete send the POST Request for file deletion
 * Insert the path to the in it. A file or a directory
 */
export class DeletionReq{
    constructor(path){
        this.path = path;
    }
}