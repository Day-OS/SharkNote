
/**
 * 
 * @param {Blob | File} file 
 * @returns {data: URL}
 */
export const toBase64 = file => new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.readAsDataURL(file);
    reader.onload = () => resolve(reader.result);
    reader.onerror = reject;
});

export async function toRAWB64(file) {
    let b64 = await toBase64(file)
    return b64.split(',')[1]
};

/**
 * 
 * @param {data: URL} dataurl 
 * @param {String} filename 
 * @returns {File}
 */
export function dataURLtoFile(dataurl, filename) {
    var arr = dataurl.split(','),
        mime = arr[0].match(/:(.*?);/)[1],
        bstr = atob(arr[arr.length - 1]), 
        n = bstr.length, 
        u8arr = new Uint8Array(n);
    while(n--){
        u8arr[n] = bstr.charCodeAt(n);
    }
    return new File([u8arr], filename, {type:mime});
}