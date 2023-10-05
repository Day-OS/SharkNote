document.addEventListener('DOMContentLoaded', () =>{
    document.getElementById('page_selection').addEventListener('change', (evt) => {
        /**
         * @type {HTMLSelectElement}
         */
        const select = evt.target;
        const index = select.selectedIndex;
        if (index > 0) {window.location.href = `/editor?page_id=${select[index].value}`}
    })
}, false);