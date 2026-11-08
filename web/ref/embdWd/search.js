/* 哂蟹齋排版器（https://vert.neocities.org/type/set/）原文件，經作者許可收錄；路徑已改為相對路徑。 */
'use strict';
var 卷搜索框 = document.getElementById("搜索框"), 歬鈕 = document.getElementById("歬一條"), 後鈕 = document.getElementById("後一條"), 搜于書鈕 = document.getElementById("搜全書鈕"), 搜于齋鈕 = document.getElementById("搜全齋鈕");
class 勾三態項 {
	constructor(選項物件,態,優先=true){
		this.選項物件 = 選項物件;
		this.態 = 態;	this.方向 = 優先?1:2;
		if (態==1){
			選項物件.checked = !優先;	選項物件.indeterminate = true;
		}
		this.偵聽事();
	}
	偵聽事(){
		var 物件 = this.選項物件, 方向 = this.方向;
		物件.addEventListener('click',()=>{
			//e.preventDefault();
			this.態 = (this.態+方向)%3;
			if (this.態 == 2){物件.checked = true;物件.indeterminate = false;}
			else if (this.態 == 1){物件.indeterminate = true;}
			else {物件.checked = 物件.indeterminate = false;}
		});
	}
}
let 異體選項 = new 勾三態項(document.getElementById("通搜否"), 1);
let 標點選項 = new 勾三態項(document.getElementById("標點符合否"), 1, false);
var content = window.parent.document.body, results, currentClass = "currentFindMatch", currentIndex = 0;
var instance = new Mark(content), 總符合數 = document.getElementById("總符合數");
function 跳轉(n){
	if (results.length==0){return;}
	let prevScrL = window.parent.檔根.scrollLeft;
	currentIndex += n;
	if (currentIndex < 0){
		currentIndex = results.length - 1;
	} else if (currentIndex > results.length - 1) {
		currentIndex = 0;
	}
	results[currentIndex].scrollIntoView({block:(n>0?"end":"start")});
	if (window.parent.檔根.scrollLeft < prevScrL){
		window.parent.檔根.scrollLeft -= window.parent.設置["字號"]*window.parent.行寬*3;
	} else if (window.parent.檔根.scrollLeft > prevScrL){
		window.parent.檔根.scrollLeft += window.parent.設置["字號"]*window.parent.行寬*2;
	}
	results[currentIndex].classList.add("currentFindMatch");
	results[(currentIndex+results.length-n)%results.length].classList.remove("currentFindMatch");
	document.getElementById("現敘號").innerHTML = String(currentIndex+1);
}
const 標點 = window.parent.標點["橫"].split("");
function 變更(輸入){
	instance.unmark({done:()=>{
		instance.mark(輸入,{"className":"MARKfounds", "acrossElements":true, "diacritics":異體選項.態,/* "exclude":標點選項.態==2?[]:[".hideCopy",".jmhgbd",".gdzhbd"],*/ "ignorePunctuation":標點選項.態==2?[]:標點,"標點符合":標點選項.態, done:()=>{
			results = content.getElementsByClassName("MARKfounds");
			總符合數.innerHTML = results.length;	document.getElementById("現敘號").innerHTML = '　';
		}})
	}});
}
卷搜索框.addEventListener('input',(e)=>{
	var 輸入 = e.target.textContent;
	變更(輸入);
});
document.getElementById("通搜否").addEventListener('click',()=>{變更(卷搜索框.textContent);});
document.getElementById("標點符合否").addEventListener('click',()=>{變更(卷搜索框.textContent);});
var 卷搜框字號 = parseFloat(window.getComputedStyle(卷搜索框,null).getPropertyValue('font-size'));
class 操作文本框 {
	constructor(文本框){
		this.縮進 = parseFloat(window.getComputedStyle(文本框,null).getPropertyValue('text-indent'));
		this.內裕 = parseFloat(window.getComputedStyle(文本框,null).getPropertyValue('padding-top'));
		this.文本框 = 文本框;
	}
	游標入框(range){
		if(range===undefined){
			const selection = window.getSelection();if (!selection.rangeCount){return false;}
			range = selection.getRangeAt(0);
		}
		const rect = range.getClientRects()[0];//webkit无選區時得不到外框
		const 下 = this.文本框.offsetParent.offsetTop;
		if (range.startOffset==0){
			this.縮進 = -下-this.內裕;this.文本框.style.textIndent = "";
		} else if (rect){
			//console.log([rect.top, this.文本框.clientHeight+this.文本框.offsetTop]);
			if(rect.top > this.文本框.clientHeight+this.文本框.offsetTop+下-this.內裕){
				this.縮進 += this.文本框.clientHeight+this.文本框.offsetTop+下-this.內裕 - rect.top;
				this.文本框.style.textIndent = String(this.縮進 -6)+"px";
			} else if(rect.top < this.文本框.offsetTop+下+this.內裕){
				this.縮進 += this.文本框.offsetTop+下+this.內裕 - rect.top;
				this.文本框.style.textIndent = String(this.縮進 +6)+"px";
			}
		}
	}
}
var 操卷搜框 = new 操作文本框(卷搜索框);
function 迻游標(字數,選=false){
	const isSupported = typeof window.getSelection !== "undefined";
	if (!isSupported){return -1;}
	const selection = window.getSelection();
	if (!selection.rangeCount){return false;}
	const range = selection.getRangeAt(0);
	let 起訖 = [selection.anchorOffset, selection.focusOffset];
	if(range.collapsed){
		if( 選 ){
			if (字數 < 0){
				range.setStart(range.startContainer, Math.max(range.startOffset+字數,0));
			} else {
				range.setEnd(range.endContainer, Math.min(range.endOffset+字數,range.endContainer.textContent.length));
			}
		} else {
			if(range.endOffset+字數<0){
				range.setEnd(range.endContainer,0);
			} else if(range.endOffset+字數>range.endContainer.textContent.length){
				range.setEnd(range.endContainer,range.endContainer.textContent.length);
			} else {
				range.setEnd(range.endContainer,range.endOffset+字數);
			}
			range.collapse(false);
		}
	} else {
		if( 選 ){
			if (起訖[0]<起訖[1]){
				range.setEnd(range.endContainer, Math.min(起訖[1]+字數,range.endContainer.textContent.length));
			} else {
				range.setStart(range.startContainer, Math.max(起訖[1]+字數,0));
			}
		}else{
			range.collapse(字數<0 ? true : false);return 0;
		}
	}
}
function 隱搜索欄(){
	window.frameElement.style.transform = "";	window.frameElement.blur();
	instance.unmark();
}
document.getElementById("隱").onclick = ()=>{隱搜索欄();};
window.oncontextmenu = (e) => {window.parent.隱選單();//e.preventDefault();
}
document.body.addEventListener('click', window.parent.隱選單);
window.addEventListener("keydown", function (event) {
	switch (event.key) {
		case "Enter":
			跳轉(1);	break;
		case "F":
		case "f":
			if (event.ctrlKey){event.preventDefault();	卷搜索框.focus();	變更(卷搜索框.textContent);}
			break;
		default:
			return;
	}
});
window.addEventListener("keyup", function (event) {
	switch (event.key) {
		case "F3":
			break;
		case "F9":
			卷搜索框.focus();	變更(卷搜索框.textContent);
			break;
		case "Escape":
			隱搜索欄();
			break;
		default:
			return;
	}
});
卷搜索框.addEventListener("keydown",(e)=>{
	switch (e.key) {
		case "Enter":
			e.preventDefault();	break;
		case "ArrowRight":
			歬鈕.style.backgroundColor = "#bbb";
			跳轉(-1);
			e.preventDefault();	break;
		case "ArrowLeft":
			後鈕.style.backgroundColor = "#bbb";
			跳轉(1);
			e.preventDefault();	break;
		case "ArrowUp":
			if(window.parent.瀏覽器["名"]!="火狐"){
				if(e.shiftKey){迻游標(-1,true);} else {迻游標(-1);}
				e.preventDefault();
			}
			操卷搜框.游標入框();
			break;
		case "ArrowDown":
			if(window.parent.瀏覽器["名"]!="火狐"){
				if(e.shiftKey){迻游標(1,true);} else {迻游標(1);}
				e.preventDefault();
			}
			操卷搜框.游標入框();
			break;
		default:
			return;
	}
});
卷搜索框.addEventListener("keyup",(e)=>{
	switch (e.key) {
		case "ArrowRight":
			歬鈕.style.backgroundColor = "";
			e.preventDefault();	break;
		case "ArrowLeft":
			後鈕.style.backgroundColor = "";
			e.preventDefault();	break;
		default:
			return;
	}
});
歬鈕.addEventListener('click',()=>{跳轉(-1);});
後鈕.addEventListener('click',()=>{跳轉(1);});
卷搜索框.addEventListener('paste', function(event){
	let paste = (event.clipboardData || window.clipboardData).getData('text');
	paste = paste.replace(/\n|\r/g,'');
	const selection = window.getSelection();
	if (!selection.rangeCount){return false;}
	selection.deleteFromDocument();
	selection.getRangeAt(0).insertNode(document.createTextNode(paste));
	this.normalize();
	event.preventDefault();
	變更(this.textContent);
});
卷搜索框.addEventListener('wheel',function(event){
	var 縮進 = parseFloat(window.getComputedStyle(this,null).getPropertyValue('text-indent'));
	var 框長 = this.clientHeight;
	縮進 -= Math.sign(event.deltaY)*30;
	if (縮進>0){
		縮進 = 0;
	}else if(縮進+卷搜框字號*this.textContent.length < 框長){
		縮進 = 框長 - 卷搜框字號*this.textContent.length;
	}
	this.style.textIndent = String(縮進)+"px";
});
if (window.parent.系統 == 'Android' || window.parent.系統 == 'iPhone'){
	document.documentElement.style.fontSize = "25px";
}
window.oncontextmenu = (e) => {
	if (e.target != 卷搜索框){e.preventDefault();}
}