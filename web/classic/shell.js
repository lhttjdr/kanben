/* 刊本・原版排版器移植：頁面主程序（改寫自哂蟹齋排版器的 main.js，經作者許可）。
   作為普通腳本載入，全局名字與原版相同；不碰頁面的計算（認瀏覽器與系統、讀數、析數、算籌樣式）
   改由 wasm（window.__kanbenWasm）完成。 */
'use strict';// 主程序.
var 檔根 = document.documentElement || document.body;
var 函 = document.getElementById("函");
var 袠 = document.getElementById("帙") || document.getElementById("袠");
function shuffleStr(str) {
  var a = str.split(""), n = a.length;
  var j, tmp;
  for(var i = n - 1; i > 0; i--) {
    j = Math.floor(Math.random() * (i + 1));
    tmp = a[i];
    a[i] = a[j];
    a[j] = tmp;
  }
  return a.join("");
}
function getBrowserInfo() {
	const [名, 版本] = __kanbenWasm.browserInfo(navigator.userAgent).split('\t');
	return {名, 版本};
}
function detectOS() {
	return __kanbenWasm.detectOs(navigator.userAgent, navigator.platform);
}
function 滑出搜索欄(事){
	var 搜索欄 = document.getElementById("搜索欄");
	if (搜索欄===null){return;}
	搜索欄.style.transform = "translateX(4em)";
	搜索欄.contentWindow.document.getElementById("搜索框").focus();
	事.preventDefault();
}
function 以鼠標橫滾(event,元素,跬=50){
	var deltaY = event.deltaY, deltaX = event.deltaX;
	if (Math.abs(deltaY)>Math.abs(deltaX)){
	  元素.scrollLeft -= 2*跬*deltaY/Math.abs(deltaY);
	}
}// IE 不行 
function keyPaging(event){
	var x = event.key;
	if (x == "PageUp"){
		檔根.scrollLeft += 500;
	}else if (x == "PageDown"){
		檔根.scrollLeft -= 500;
	}else if (x == "Home"){
		檔根.scrollLeft = 檔根.scrollWidth;window.scrollTo(檔根.scrollWidth, 0);
	}else if (x == "End"){
		//window.scrollTo(0, 0);
		檔根.scrollLeft -= 檔根.scrollWidth
	}else if (x == "F9" || (event.code == "KeyF" && event.ctrlKey)){
		滑出搜索欄(event);
	}
}
if(!decodeURIComponent(window.location.pathname.replace(/\/$/,'')).endsWith('/萬年夏曆')){
	檔根.addEventListener('wheel',function(evt){以鼠標橫滾(evt,this);});
	document.body.addEventListener('keydown',keyPaging);
}
var 行寬 = 1.6, 跨註行寬 = 1;
var 瀏覽器 = getBrowserInfo();
var 系統 = detectOS();
var 設置 = {"字號": 25, "字號增減": 0, "頂列顯隱": true, "選單顯隱":false};
var 指觸 = {"pgX": 0, "pgY": 0};
var contentBlk = document.getElementById("content_blk");
function adaptM(th,卷數多=false) {
	if( ['Android','iPhone','iPad','鴻蒙'].indexOf(系統)>=0 && 瀏覽器.名 != '火狐'){
		let mwkb = document.createElement('link');// mobile webkit bug
		mwkb.rel = "stylesheet"; mwkb.href = "../../nongeckobug.css"
		document.head.appendChild(mwkb);
	}
	var 劉海元素 = document.getElementById("劉海");if(劉海元素===null){劉海元素 = document.getElementById("top_banner");}
	檔根.style.setProperty('--劉海高', th);
	if (window.innerWidth/window.innerHeight <= 0.9){
		if ( 瀏覽器.名 == '火狐') {
			設置["字號增減"] = 3;
		} else if(document.getElementById('miun')!==null){
			document.getElementById('miun').style.paddingRight = "7px";
		}
		
		檔根.style.setProperty('--基礎字號', String(設置["字號"]+設置["字號增減"])+"px");// 檔根.style.fontSize = String(設置["字號"]+設置["字號增減"])+"px";
		// contentBlk.style.margin = "60px 0 0 0";
		// contentBlk.style.paddingTop = "10px";//String(th == 1 ? 2.25*th : 0)+"em";
		// try {
		//   document.getElementById('right_row').style.display = document.getElementById('left_row').style.display = "none";
		// } catch (e){/*書目䈎*/}
		// 劉海元素.style.height = String(4.5*th)+"em";
		if( 卷數多 ){劉海元素.classList.add("手機劉海籤多者");}
		else {劉海元素.classList.add("手機劉海籤少者");}
		var dr = document.createElement("DIV");dr.id = "drager";document.body.appendChild(dr);
		// document.getElementById('drager').style.top = String(4.4*th)+"em";
		if (th==0) {劉海元素.style.display = document.getElementById('drager').style.display = "none";}

		var shtop = function (event) {
			if ( 設置.頂列顯隱 ) {
				劉海元素.style.top = "-"+ String(4.5*th)+"em";
				document.getElementById('drager').style.top = "0px";
				document.getElementById('drager').style.opacity = 1;
			} else {
				劉海元素.style.top = 0;
				document.getElementById('drager').style.top = String(4.4*th)+"em";
				//document.getElementById('drager').style.borderBottom = "15px solid rgba(153, 22, 33, 0)";
				document.getElementById('drager').style.opacity = 0.1;
			}
			設置.頂列顯隱 = !設置.頂列顯隱;
		};
		document.getElementById('drager').addEventListener('touchend', shtop, false);
		contentBlk.addEventListener('touchend', ()=>{if( 設置.頂列顯隱 ){shtop();}}, false);
		/*document.body.addEventListener('touchmove', function (event) {
			var delta = {"X":0, "Y":0};
			if( 指觸["pgY"]!=0) {
				delta.Y = event.touches[0].pageY - 指觸["pgY"];
				delta.X = event.touches[0].pageX - 指觸["pgX"];
				if (Math.abs(delta.Y) > 40 && Math.abs(delta.Y) > Math.abs(delta.X)*1.4) {
					shtop();
				}
			}
			指觸["pgX"]= event.targetTouches[0].pageX; 指觸["pgY"]= event.targetTouches[0].pageY;
		}, false);*/
	// } else if( 系統 == 'iPad'){
	//   檔根.style.setProperty('--基礎字號', String(設置["字號"]+設置["字號增減"]-10)+"px");
	}
}
if ( 瀏覽器.名 == "火狐" && Number(瀏覽器.版本.replace(/\..+/,'')) < 63) {
  if(window.location.href.replace(/^.+\/(.+?)\.html/,'$1')!= 'browsOld'){
    window.location.replace("../../../ref/browsOld.html");
  }
} else if (瀏覽器.名 == "IE") {
  window.location.replace("../../../ref/browsChng.html");
} else if (瀏覽器.名 == "微信" || 瀏覽器.名 == "QQ") {
  let 全遮板 = document.createElement("div");
  全遮板.style.cssText = "position:fixed; z-index:900; height:100vh; width:100vw; top:0; left:0; background-color:#333; color:#ccc; font-size:8vh; line-height:100vw; letter-spacing:2vh; text-align:center";
  全遮板.innerHTML = '請以瀏覽器啓視';
  document.body.appendChild(全遮板);
} else {
  檔根.style.setProperty('--基礎字號', "15px");// 檔根.style.fontSize = "15px";//1.25*0.75 要整 5px
}// 其他瑕疵.

var lastTouchEnd = 0;
function 隱選單(){
  let 選單 = document.getElementById('ctxmenu');
  if( 選單 !== null){
    選單.style.display = "none";
  }
}
if (contentBlk!=undefined){
contentBlk.addEventListener('touchend', function (event) {
  var now = Date.now(), 選單 = document.getElementById('ctxmenu');
  if( 選單 !== null && 設置.選單顯隱){選單.style.display = "none";設置.選單顯隱 = false;}
  if (now - lastTouchEnd <= 300) {
    event.preventDefault();
    if( 選單 !== null){
      if( !設置.選單顯隱){
        選單.style.display = "block";
        選單.style.right = "1vw";選單.style.top = 0;
        設置.選單顯隱 = true;
      }
      //設置.選單顯隱 = !設置.選單顯隱;
    } else {
      設置.字號 += 5;if( 設置.字號 >30){ 設置.字號 = 20;}
      if( 系統 == 'iPad'){
        檔根.style.setProperty('--基礎字號', String(設置["字號"]+設置["字號增減"]-10)+"px");// 檔根.style.fontSize = String(設置["字號"]+設置["字號增減"]-10)+"px";
      } else {
        檔根.style.setProperty('--基礎字號', String(設置["字號"]+設置["字號增減"])+"px");//檔根.style.fontSize = String(設置["字號"]+設置["字號增減"])+"px";
      }
      度行();增刪絲欄();
    }
  }
  lastTouchEnd = now;
}, false);
}
function 整數籌(整數,个位縱=true){
	const [數, 類, 寬窄, 負號] = __kanbenWasm.rodStyle(整數, 个位縱).split('\t');
	return {數: typeof 整數 === 'number' ? Number(數) : 數, 類, 寬窄: Number(寬窄), 負號};
}
var 正文字號 = 30, mhgs = 20.5, 每行格數 = 20, 行尾空白長 = 0.5;
function 度行(){
  if (document.getElementById('miun')!==null){
    const 目標行尾長 = 0.2;
    document.getElementById('miun').style.height = "unset";
    正文字號 = parseFloat(window.getComputedStyle(document.getElementById('miun'),null).getPropertyValue('font-size'));
    mhgs = document.getElementById('miun').clientHeight / 正文字號;
    每行格數 = Math.floor(mhgs);
    行尾空白長 = mhgs - 每行格數;
    if(mhgs>3){
      mhgs = 每行格數 + 目標行尾長;行尾空白長 = 目標行尾長;
      document.getElementById('miun').style.height = String(mhgs)+"em";document.getElementById('gaeg').style.height = String(mhgs+0.4)+"em";
    } else {
      document.getElementById('gaeg').style.height = "";
    }
  }
}
function 增刪絲欄() {
  let 絲欄 = document.getElementById("jiusilan");
  if (絲欄 === null){return;}
  let 現有行數 = 絲欄.childNodes.length;// 或.lastChild.textContent
  let 所需行數 = Math.round(document.getElementById('miun').clientWidth/正文字號/行寬)+1;
  if ( 現有行數 == 所需行數 ){return;}
  var newhang, newhb;
  if ( 現有行數 < 所需行數 ){// 增+
    let fragment = new DocumentFragment(), 行號;
    for (let hang = 現有行數+1; hang <= 所需行數; hang++){
      行號 = 整數籌(hang,false);
      newhang = document.createElement("div");newhang.className = "jienghangsen";
      newhb = document.createElement("div");newhb.className = "hangbiau lrtb "+ 行號["類"];
			if(行號["寬窄"]>0){
				newhb.style.fontStretch = String(行號["寬窄"])+"%";
			}
      newhb.appendChild(document.createTextNode(hang));newhang.appendChild(newhb);
      fragment.appendChild(newhang);
    }
    絲欄.appendChild(fragment);
  } else {// 減-
    for (let hang = 現有行數; hang > 所需行數; hang--){
      絲欄.removeChild(絲欄.lastChild);
    }
  }
}
//var tt0 = Date.now();
function 測字距(樣式或元素){
  var testdiv, h;
  if(樣式或元素.substring(0,1)=='.'){
    testdiv = document.createElement("DIV");
    testdiv.className = 樣式或元素.substring(1);
  } else {
    testdiv = document.createElement(樣式或元素);
  }
  testdiv.id = "measr";
  testdiv.innerHTML = '甲乙丙丁戊己庚辛壬癸';
  document.body.appendChild(testdiv);
  h = parseFloat(window.getComputedStyle(document.getElementById('measr'),null).getPropertyValue('height'));
  document.body.removeChild(testdiv);
  return 1-h/400;
}
var 調距 = {仿宋: 測字距(".f3s5"), 楷: 測字距(".krai3"), 明: 測字距(".mieng2"), 夾注: 測字距(".grebdriuo"), 注中注: 測字距(".DRinDR"), 注中小字: 測字距(".DRinDRhf"), 敘: 測字距(".sio4"), 跋: 測字距(".buad8"), 正文: 測字距("P")/*, 題: [測字距("H1"),測字距("H2"),測字距("H3")]*/};
//console.log(`逐个測字距耗時：${Date.now()-tt0}`);

document.body.addEventListener('contextmenu', e => {
  if( 系統 == 'Android' || 系統 == 'iPhone'){return;}
  let 選單 = document.getElementById('ctxmenu');
  if( 選單 !== null){
    if (e.target != document.getElementById("outPDFpan") && e.target != document.querySelector("a")){
      e.preventDefault();
      選單.style.display = "block";
      if (e.clientX < 選單.clientWidth){
        選單.style.left = e.clientX +'px';
      } else {
        選單.style.left = (e.clientX-選單.clientWidth) +'px';
      }
      if (e.clientY > window.innerHeight - 選單.clientHeight){
        選單.style.top = (e.clientY-選單.clientHeight) +'px';
      } else {
        選單.style.top = e.clientY +'px';
      }
    } else if (e.target == document.querySelector("a")){
      選單.style.display = "none";
    }
  }
});
document.body.addEventListener('click', 隱選單);
function 隱此物(e){
  let elmt = e.target || e.srcElement;
  elmt.style.display = "none";
}
window.onload = (e) => {
  if (document.getElementById('outPDFpan')!==null){
    document.getElementById('outPDFpan').addEventListener('click',隱此物);
  }
};
var 鏈題元素 = 'h2';
function 滾到節(n){
  if (鏈題元素 == '〔〖〗〕'){
    鏈題元素 = '.mh6grug';
  }
  var 諸題 = document.querySelectorAll(鏈題元素);
  if (諸題[n-1]===undefined){return;}
  let prevScrL = 檔根.scrollLeft;
  if (諸題[n-1].parentElement.tagName == "SECTION"){
    諸題[n-1].parentElement.scrollIntoView();
  } else {
    諸題[n-1].scrollIntoView();
  }
  if (瀏覽器["名"]!='火狐'){return;}
  if (檔根.scrollLeft < prevScrL){
		檔根.scrollLeft -= 設置["字號"]*行寬*6;
	} else if (檔根.scrollLeft > prevScrL){
		檔根.scrollLeft += 設置["字號"]*行寬*4;
	}
}
function 重排(domid="miun"){
	度行();
	調整雙行夾注(domid);寫旁句();
	對齊眉批();增刪絲欄();
}
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
function 貼純文本(框,event){
	let paste = (event.clipboardData || window.clipboardData).getData('text');
	paste = paste.replace(/\n|\r/g,'');
	const selection = window.getSelection();
	if (!selection.rangeCount){return false;}
	selection.deleteFromDocument();
	selection.getRangeAt(0).insertNode(document.createTextNode(paste));
	框.normalize();
	event.preventDefault();
	//變更(this.textContent);
}
/*if (( 系統 == 'Android' || 系統 == 'iPhone' || 系統 == '鴻蒙')&& 瀏覽器.名 != '火狐') {
  檔根.style.writingMode = "initial";
  檔根.style.direction = "rtl";
  if (函 == null){
    document.body.innerHTML = '<div id="函">'+document.body.innerHTML+'</div>';
  }
}*/
var 標點 = {
    橫: '，、。．：；？！·《》〈〉「」『』【】〖〗（）［］〔〕…〪〭〫〬_‹›',//．替換成・
    縱: '︐︑︒・︓︔︖︕･︽︾︿﹀﹁﹂﹃﹄︻︼︵︶﹇﹈︹︺︙¯ˇˋˆ︳﹤﹥'// ̄́̌̀̂.
};
const 數 = {
  碼子:['〇','〡','〢','〣','〤','〥','〦','〧','〨','〩'],
  字:['〇','一','二','三','四','五','六','七','八','九'],
  大寫:['零','壹','貳','叄','肆','伍','陸','柒','捌','玖'],
  古大寫:['霝','弌','弍','弎','肆','伍','陸','漆','捌','玖'],
  縱筭籌:['〇','𝍩','𝍪','𝍫','𝍬','𝍭','𝍮','𝍯','𝍰','𝍱'],
  橫筭籌:['〇','𝍠','𝍡','𝍢','𝍣','𝍤','𝍥','𝍦','𝍧','𝍨'],
  十進級:['十','百','千'],
  大數級:['萬','億','兆','京','垓','秭','穰','溝','澗','正','載'],
  小數級:['分','釐','毫','絲','忽','微','纖','沙','塵','埃','渺','漠','糢'],// 糢出自《宛署雜記·繇賦》
  諸十合文:['十','廿','卅','卌','五十','六十','七十','八十','九十'],
  讀百以內合文:function(an){
    return __kanbenWasm.readUnder100(an);
  },
  讀十萬以內:function(an){
    return __kanbenWasm.readUnder100k(an);
  }
};
function 析整數(讀法){
	return __kanbenWasm.parseHanNumber(讀法);
}
if(袠!==null){
	袠.addEventListener('click',function(){
    this.style.opacity = "0";
    setTimeout(()=>{this.style.display="none";}, 1000);
  });
}
const 命令 = {
  格式:['敘','序','跋','結語','署名','落款','譔者','撰者','卷名','目次','篇目','篇名','分層','層','減字','半字','工尺','律呂','陣','分數'],
  字符:['頓首','招財進寶','學好孔孟']
};