const audio = new Audio('static/sounds/blup.ogg')
document.addEventListener('DOMNodeInserted', (e)=>{
    
    if (!e.target.classList.contains("toast")) {return}
    /**
     * @type {Element}
     */
    const notification = e.target;
    notification.addEventListener("hidden.bs.toast", ()=>{
        notification.remove();
    })
    notification.addEventListener("shown.bs.toast", ()=>{
        audio.play()
    })
    new bootstrap.Toast(notification).show()
})



/*
export class Notification {
    constructor(title, content){
        this.title = title
        this.content = content
    }
    show(){
        const audio = new Audio('static/sounds/blup.ogg')
        audio.play();
        const containter = document.getElementById("notification-container");
        const notification_element = document.getElementById("notification-template").content.firstElementChild.cloneNode(true);
        notification_element.addEventListener('hidden.bs.toast', () => {
            console.log("aaa");
          })
          
        let notification = new bootstrap.Toast(notification_element)
        console.log(notification_element);
        console.log(notification._element)
        containter.append(notification._element)
        notification.show()
    }
}

document.addEventListener('DOMContentLoaded', () =>{
    new Notification("asd", "sdf").show()
    


    
}, false);

*/