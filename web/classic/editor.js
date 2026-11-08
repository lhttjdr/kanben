/* 刊本・原版排版器移植：編輯器（改寫自哂蟹齋排版器的 ts.js，經作者許可）。
   作為普通腳本載入；語法著色與換行計數由 wasm（window.__kanbenWasm）完成。 */
'use strict';
var 編輯器 = document.getElementById("編輯器"), 富編輯器 = document.getElementById("富編輯器");
function 在何段(文本, 字符敘){
	return __kanbenWasm.lineOf(文本, 字符敘);
}
class 編輯純文本 {
	constructor(純文本框, 富文本框){
		this.純文本框 = 純文本框;this.富文本框 = 富文本框;
	}
	移游標(){
		const 游標所在字符 = this.純文本框.selectionEnd;
		const 游標所在段 = 在何段(this.純文本框.value, 游標所在字符);
		this.純文本框.dataset.caretAt = 游標所在段;
	}
	渲染(){
		this.富文本框.innerHTML = __kanbenWasm.highlight(this.純文本框.value);
		var 諸題段 = this.富文本框.getElementsByClassName("edit_title");
		for (let 題段 of 諸題段){
			題段 = 題段.nextElementSibling;
			while (題段 && 題段.textContent!='' && !題段.classList.contains("edit_title")){
				題段.classList.add("edit_title");
				題段 = 題段.nextElementSibling;
			}
		}
	}
	備(){
		this.純文本框.addEventListener('wheel', (evt)=>{以鼠標橫滾(evt,this.純文本框);});
		this.純文本框.addEventListener('scroll', ()=>{
			this.富文本框.scrollLeft = this.純文本框.scrollLeft;
		});
		if(瀏覽器["名"]=='鉻' && Number(瀏覽器["版本"].split('.')[0])<120){
			this.純文本框.placeholder = this.純文本框.title = "此瀏覽器太舊。請陞級到版本 120.0.6067 以上，以實現縱排編輯。";
			this.富文本框.style.opacity = 0;
			this.純文本框.style.cursor = "auto"; this.純文本框.style.color = "var(--主字色)"; this.純文本框.style.backgroundColor = "var(--紙色)";
		}
		this.渲染();
		this.純文本框.focus();this.移游標();
		this.純文本框.dataset.caretAt = 在何段(this.純文本框.value, this.純文本框.selectionEnd);
		this.純文本框.addEventListener('input', ()=>{this.渲染();});
		this.純文本框.addEventListener('selectionchange', ()=>{this.移游標();});
	}
}

document.getElementById("編譯鈕").addEventListener('click', ()=>{單卷.排版();});
document.getElementById("結果").addEventListener('wheel', function(evt){以鼠標橫滾(evt,this);});
var 單卷 = new 哂蟹卷({源: 編輯器});
單卷.排版();
document.getElementById("打印鈕").addEventListener('click', ()=>{window.print();});
document.getElementById("下載鈕").addEventListener('click', ()=>{
	const 文本 = 編輯器.value;
	const blob = new Blob([文本], {type: "text/plain"});
	let 鏈 = document.createElement('a');
	鏈.download = "哂蟹齋排版文本.txt";
	鏈.href = URL.createObjectURL(blob);
	鏈.click();
	URL.revokeObjectURL(鏈.href);
});
document.getElementById("切䈎鈕").addEventListener('click', ()=>{
	卷䈎互轉();
});

function __移游標(文本框=編輯器){
	const 游標所在字符 = 文本框.selectionEnd;
	const 游標所在段 = 在何段(文本框.value, 游標所在字符);
	// if (富編輯器.children[游標所在段] && !富編輯器.children[游標所在段].classList.contains("crntLine")){
	// 	let 當歬段 = 富編輯器.querySelector('.crntLine');
	// 	if(當歬段!==null){當歬段.classList.remove("crntLine");}
	// 	富編輯器.children[游標所在段].classList.add("crntLine");
		文本框.dataset.caretAt = 游標所在段;
	// }
}
var 用戶編輯 = new 編輯純文本(編輯器, 富編輯器);
用戶編輯.備();