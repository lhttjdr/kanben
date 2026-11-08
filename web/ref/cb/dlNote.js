/* 哂蟹齋排版器（https://vert.neocities.org/type/set/）原文件，經作者許可收錄；路徑已改為相對路徑。 */
'use strict';
const 冓 = {"⿰":2,"⿲":3,"⿱":2,"⿳":3,"⿴":2,"⿵":2,"⿶":2,"⿷":2,"⿸":2,"⿹":2,"⿺":2,"⿻":2,"⿼":2,"⿽":2,"⿾":1,"⿿":1,"４":4,"５":5,"Ｔ":3,"Ｙ":3,"Ａ":1,"＋":1,"Ｗ":1};
var 補遺字 = [], 拼字 = [];
function 遞歸構字(構字式, 到=0){
	var 構成 = '';
	if (到 >= 構字式.length){
		return [構成, 到];
	}
	if (Object.keys(冓).indexOf(構字式[到]) >= 0){
		let 件數 = 冓[構字式[到]];
		構成 = 構字式[到];
		for (let 件第=0; 件第<件數; 件第++){
			到 += 1;
			let 件_到 = 遞歸構字(構字式, 到)
			構成 += 件_到[0];到 = 件_到[1];
		}
		return ['☯', 到];
	} else {
		return [構字式[到], 到];
	}
}
function 構字(構件){
	var 成字 = '';const 結構 = 構件.substring(0,1);
	if (結構 == '⿰' && 構件.length == 3){
		成字 = '<span class="cc_LR_L">'+ 構件.substring(1,2) +'</span><span class="cc_LR_R">'+ 構件.substring(2,3) +'</span>';
	} else if (結構 == '⿱' && 構件.length == 3){
		成字 = '<span class="cc_TB_T">'+ 構件.substring(1,2) +'</span><span class="cc_TB_B">'+ 構件.substring(2,3) +'</span>';
	} else if (結構 == '⿸' && 構件.length == 3){
		成字 = '<span class="cc_TLBR_TL">'+ 構件.substring(1,2) +'</span><span class="cc_TLBR_BR">'+ 構件.substring(2,3) +'</span>';
	} else {
		成字 = 構件.replace(/[\u2FF0-\u2FFB]/g,'');
	}
	成字 = '<i class="charComp">'+ 成字 +'</i>';
	return 成字;
}
function punctuateVH(樣式類, tf, ctxt) {
	var x;
	if (tf) {
		拼字 = ctxt.match(/\[\[\?.+?¿\]\]/g);
		ctxt = ctxt.replace(/\[\[\?.+?¿\]\]/g,'塀');
		補遺字 = ctxt.match(/[\uD800-\uDFFF][\uD800-\uDFFF]/g);
		ctxt = ctxt.replace(/[\uD800-\uDFFF][\uD800-\uDFFF]/g,'凞');
		x = ctxt.match(/\(\(=(.+?)=\)\)/g);
		if(x!==null){
			for (var i=0; i<x.length; i++){//[].forEach.call(x, function (xi) {
				for (var d = 0; d < 標點.橫.length; d++) {
					x[i] = x[i].replace(new RegExp(標點.橫.charAt(d),'g'), 標點.縱.charAt(d));
				}
				x[i] = x[i].replace(/\(\(=(.+?)=\)\)/g,'<small class="'+ 樣式類 +'">$1</small>');
				ctxt = ctxt.replace(/\(\(=.+?=\)\)/,x[i]);
			}
		}
		x = ctxt.match(/““=(.+?)=””/g);
		if(x!==null){
			for (var i=0; i<x.length; i++){
				for (var d = 0; d < 標點.橫.length; d++) {
					x[i] = x[i].replace(new RegExp(標點.橫.charAt(d),'g'), 標點.縱.charAt(d));
				}
				x[i] = x[i].replace(/““=(.+?)=””/g,'<dd class="'+ 樣式類 +'">$1</dd>');
				ctxt = ctxt.replace(/““=(.+?)=””/,x[i]);
			}
		}
		return ctxt;
	} else {
		x = document.getElementsByClassName(樣式類);var xt;
		[].forEach.call(x, function (item) {
				xt = item.innerHTML;
				for (var d = 0; d < 標點.縱.length; d++) {
						xt = xt.replace(new RegExp(標點.縱.charAt(d),'g'), 標點.橫.charAt(d));
				}
				if (調距["夾注"]!=-0.25){item.style.letterSpacing = String(調距["夾注"]+0.5)+"em";}
				item.innerHTML = xt;
		});
		if( 補遺字 === null && 拼字 === null){return;}
		x = document.getElementById(ctxt).innerHTML;// 此ctxt實為cbox
		if( 補遺字 !== null){
			for (var i=0; i<補遺字.length; i++) {
				x = x.replace(/凞/,'<span class="extGK_FS">'+補遺字[i]+'</span>');
			}
		}
		if( 拼字 !== null){
			for (var i=0; i<拼字.length; i++) {
				拼字[i] = 拼字[i].replace(/[\[\?¿\]]/g,'');
				x = x.replace(/塀/, 構字(拼字[i]));
			}
		}
		document.getElementById(ctxt).innerHTML = x;
	}
}

function 寫眉批(cvTXT,ttID){/* (?〈眉批文〉樣式敘號¿) */
	var 批楎 = cvTXT.match(/\(\?〈.+?〉\d*¿\)/g), 批文, 樣式號;
	document.getElementById(ttID).innerHTML = '';if(批楎===null){return cvTXT;}
	var x = cvTXT.replace(/\(\?〈(.+?)〉\d*¿\)/g,'<span class="rsvnote"></span>');
	for (var i = 0; i < 批楎.length; i++) {
		批文 = document.createElement("DIV");批文.id = "mp"+String(i);批文.innerHTML = 批楎[i].replace(/\(\?〈|〉\d*¿\)/g,'');
		樣式號 = 批楎[i].match(/\d*(?=¿\))/)[0];批文.className = "miipei"+(樣式號 ? " mpstl"+樣式號 : "");
		document.getElementById(ttID).appendChild(批文);
	}
	return x;
}

function 對齊眉批(){
	var 批文 = document.getElementsByClassName("miipei"), 批楎;
	try {
		if (當歬卷 < 0){
			批楎 = document.getElementById("miug").getElementsByClassName("rsvnote");
		} else {
			批楎 = document.getElementById("inMain").getElementsByClassName("rsvnote");
		}
	} catch(e){
		批楎 = document.getElementsByClassName("rsvnote");
	}
	var 卷右邊寬 = parseFloat(window.getComputedStyle(document.getElementById('content_blk'),null).getPropertyValue('margin-right'))+parseFloat(window.getComputedStyle(document.getElementById('content_blk'),null).getPropertyValue('padding-right'));
	for (var i = 0; i < 批楎.length; i++) {
			批楎[i].innerHTML = '';
			if(批楎[i].offsetParent==document.getElementById("inMain") || 批楎[i].offsetParent==document.getElementById("miug")){
				批文[i].style.right = String(document.documentElement.scrollWidth- 批楎[i].offsetLeft- 卷右邊寬 *2 - 正文字號 *3)+"px";
			} else {
				批文[i].style.right = String(document.documentElement.scrollWidth- 批楎[i].offsetLeft- 批楎[i].offsetParent.offsetLeft - 卷右邊寬 *2 - 正文字號 *3)+"px";
			}
	}

}
function 列表(txt) {
	var vsl = txt.match(/(\r\n|\n\r|\r|\n)\[ \](.|\r\n|\n\r|\r|\n)+?(\r\n|\n\r|\r|\n)(?!\[ \])/gm), vslHTM = new Array(vsl.length);
	for (let i = 0; i < vsl.length; i++){
		vslHTM[i] = vsl[i].replace(/\r\n|\n\r|\r|\n/g,'</li><li>');
	}
}
function 畫表格(txt) {
  var tbts = txt.match(/┌(.|\r\n|\n\r|\r|\n)+?┘/gm);if(tbts===null){return txt;}
  var 表題 = txt.match(/┌.+?┐/g);//(/┌─*[^─\s]*─*┐/g);
  var 表行 = [];var 跨行數 = 1, 跨列數 = 1;var ss,tt;
  var tabHTM = [];
  for (let i = 0; i < tbts.length; i++){
    表題[i] = 表題[i].replace(/[┌─┐]/g,'');
    //if (表題[i]) {tbts[i] = tbts[i].replace(/┌.+?┐/g,'<caption>'+表題[i]+'</caption>');}
    表行 = tbts[i].split(/\r\n|\n\r|\r|\n/);tabHTM = tbts[i].split(/\r\n|\n\r|\r|\n/);//match(/├.+?│/gm);
    tbts[i] = '';
    for (let h = 1; h < 表行.length-1; h++){
      表行[h] = 表行[h].trim();tabHTM[h] = 表行[h].split(/[├┼│┤]/);
      tabHTM[h].shift();
      if( 表行[h].substring(0,1)=='├'){
        ss = 表行[h].match(/[├┼│┤]/g);tt = ss;
        for (let ll = 1; ll < ss.length; ll++){
          if (!tt[ll-1]){continue;}
          if(ss[ll]=='├'){
            tt[ll-1] = 'td';
            tabHTM[h][ll-1] = '<td>'+tabHTM[h][ll-1]+'</td>';
          } else if(ss[ll]=='┼'||ss[ll]=='┤'){
            tt[ll-1] = 'th';
            tabHTM[h][ll-1] = '<th>'+tabHTM[h][ll-1]+'</th>';
          } else if(ss[ll]=='│'){
            if(ll==ss.length-1){
              tt[ll-1] = 'td';
              tabHTM[h][ll-1] = '<td>'+tabHTM[h][ll-1]+'</td>';
            } else {
              tt[ll] = '';
              //colspan
              tabHTM[h][ll] = '';
            }
          }
        }
        //tt = 表行[h].split(/[├┼│┤]/);
        //表行[h] = 表行[h].replace(/├/,'<td>');
        //表行[h] = 表行[h].replace(/├/g,'</td><td>');
        //表行[h] = 表行[h].replace(/┼/g,'</th><td>');
      } else if( 表行[h].substring(0,1)=='┼'){
        console.log('第 ' + h + ' 行以┼開頭');
      } else if( 表行[h].substring(0,1)=='└'){
        console.log('第 ' + h + ' 行以└開頭');
      }
      表行[h] = '<tr>'+ tabHTM[h].join('')+'</tr>';
      tbts[i] += 表行[h];//console.log(表行[h]);
    }
    if (表題[i]) {tbts[i] = '<caption>'+表題[i]+'</caption>'+tbts[i];}
    //console.log(tbts[i]);
    txt = txt.replace(/┌(.|\r\n|\n\r|\r|\n)+?┘/m,'<table>'+tbts[i]+'</table>');
    
  }// 每个表格.
  return txt;
}
function 渲染段(txt) {
  var ps = txt.match(/>(\s*?)\(\{.+?\}\)/g); var clrs, temp;
  if (ps !== null) {
    for (var i = 0; i < ps.length; i++) {
      ps[i] = ps[i].replace(/[\(\{\}\)]/g,'').replace(/>(\s*?)/,'');
      ps[i] = ps[i].replace(/字色:([^,]+?)($|;)/,'color:var(--$1);');
      clrs = ps[i].match(/字色:(.+?)($|;)/);
      if (clrs !== null) {clrs = clrs[0].replace(/字色:|;/g,'').split(',');temp = '';
        for (var j = 0; j < clrs.length; j++) {
          temp += ',var(--'+ clrs[j]+') calc(var(--行寬)*'+ String(j)+') calc(var(--行寬)*'+ String(j+1)+')';
        }
        ps[i] = ps[i].replace(/字色:(.+?)($|;)/,'color:transparent; background:repeating-linear-gradient(to left'+ temp +'); -webkit-background-clip:text; background-clip:text;');
      }
      //ps[i] = ps[i].replace(/齊行:尾($|;)/,'text-align:right;');
      ps[i] = ps[i].replace(/⇥($|;)/,'text-align:right; margin-bottom:'+String(行尾空白長)+'em$1');
      ps[i] = ps[i].replace(/首縮進:(\d)($|;)/,'margin-top:$1em;');ps[i] = ps[i].replace(/尾縮進:(\d)($|;)/,'padding-bottom:$1em;');
      ps[i] = ps[i].replace(/(\s*?)/,'');
      txt = txt.replace(/>(\s*?)\(\{.+?\}\)/,' style="'+ps[i]+'">');
    }
  }
  return txt;
}
function shTailCmtHr(n,event){
	var 顯示框 = document.getElementById('隨顯校注');
	if(n<0){
		if(顯示框!==null){顯示框.style.display="none";}
		return;
	}
	if(event){
		var el = document.getElementById("pCmtA"+String(n));
		if(el===null){return;}
		if(顯示框===null){
			顯示框 = document.createElement("DL");顯示框.id = "隨顯校注";document.body.appendChild(顯示框);
		}
		var 條 = el.childNodes, tm;
		顯示框.innerHTML = '';
		for(let t=1; t<條.length; t++){
			if(條[t].tagName=="DT"){
				tm = 條[t].innerHTML;tm = tm.replace(/<span[^<>]+?><\/span>/g,'');
				tm = tm.replace(/(\s|")gdzhbd(?=\s|")/g,'$1jmhgbd').replace(/(\s|")zzbd(Sm|Md|Tl)(?=\s|")/g,'$1hgbdSys$2').replace(/(\s|")zzbdQt(?=\d\s|\d")/g,'$1hgbdSysQt');
				顯示框.innerHTML += '<dt>'+ tm +'</dt>';
			}else if(條[t].tagName=="DD"){
				tm = 條[t].innerHTML;tm = tm.replace(/<span[^<>]+?><\/span>/g,'');
				tm = tm.replace(/(\s|")gdzhbd(?=\s|")/g,'$1jmhgbd').replace(/(\s|")zzbd(Sm|Md|Tl)(?=\s|")/g,'$1hgbdSys$2').replace(/(\s|")zzbdQt(?=\d\s|\d")/g,'$1hgbdSysQt');
				顯示框.innerHTML += '<dd>'+ tm +'</dd>';
			}
		}
		if(顯示框.innerHTML.length>0){
			顯示框.style.display="block";
			顯示框.style.left = (event.clientX>200 ? String(event.clientX-100) : "0")+"px";//console.log(`鼠標縱 ${event.clientY}，橫${event.clientX}`);
		}
	} else{console.log((event.target || event.srcElement).tagName);}
}
function 尾注(txt){
	if (txt.match(/\(\(#.+?#\)\)/g) === null){return txt;}
	var x = txt.match(/\(\(##.+?##\)\)/g), cmnts = '';
	if (x !== null){
		cmnts = '<ol class=gCmnt>'
		for (let i=0; i<x.length; i++){
			x[i] = x[i].replace(/^\(\(##|##\)\)$/g,'');
			cmnts += '<li>'+ x[i]+'</li>';
			txt = txt.replace(/\(\(##.+?##\)\)/,'<small class="gCmtN">'+String(i+1)+'</small>');// 他日改用筭籌.
		}
		cmnts += '</ol>';
		txt += '<br>'+ cmnts;
	}
	const 後註單位 = ['section','blockquote','ol','ul','p'];
	var ps, 後註編號 = 0, __ps_r = '';//節段首正則式, 整節段正則式
	x = txt.match(/\(\(#.+?#\)\)/g), cmnts = '';
	if (x !== null){
		for (let d=0; d<後註單位.length; d++){
			ps = txt.match(RegExp('<'+後註單位[d]+'[\\s>].+?</'+後註單位[d]+'>','g'));
			if(ps===null){continue;}
			//節段首正則式 = RegExp('<'+後註單位[d]+'[\\s>]','');整節段正則式 = RegExp('<'+後註單位[d]+'[\\s>].+?</'+後註單位[d]+'>','');
			for (let r=0; r<ps.length; r++){
				x = ps[r].match(/\(\(#.+?#\)\)/g);
				if(x===null){/*txt = txt.replace(節段首正則式,'< '+後註單位[d]+' >');*/continue;}
				__ps_r = ps[r];// 原樣，帶(#)
				cmnts = '<ol class="pCmnt">';
				for (let i=0; i<x.length; i++){
					x[i] = x[i].replace(/^\(\(#|#\)\)$/g,'');
					cmnts += '<li id="pCmtA'+String(++後註編號)+'"><span class="pCmtAfN"><i class="hideCopy">〘</i>'+ 數.讀百以內合文(i+1)+'<i class="hideCopy">〙</i></span>'+ x[i]+'</li>';
					ps[r] = ps[r].replace(/\(\(#.+?#\)\)/,'<i class="hideCopy">〘</i><small class="pCmtN" onmouseover="shTailCmtHr('+String(後註編號)+',event)" onmouseout="shTailCmtHr(-1)"><span>'+ 數.讀百以內合文(i+1)+'</span></small><i class="hideCopy">〙</i>');
				}
				cmnts += '</ol>';
				ps[r] += cmnts;//ps[r] = ps[r].replace(節段首正則式,'< '+後註單位[d]+' >');
				txt = txt.replace(__ps_r,ps[r]);
			}
			//txt = txt.replace(RegExp('< '+後註單位[d]+' >','g'),'<'+後註單位[d]+'>');
		}
	}
	return txt;
}
function 分節(txt){
	txt = txt.replace(/<p>\{◎◎§/g,'<section><p>');
	txt = txt.replace(/\{◎◎§<\/p>/g,'</p><section>');
	txt = txt.replace(/§〷〷\}<\/p>/g,'</p></section>');
	txt = txt.replace(/<p>§〷〷\}/g,'</section><p>');
	txt = txt.replace(/\{◎◎§(.+?)§〷〷\}/g,'<section>$1</section>');
	return txt;
}
function 引詩文(txt){
	var qt1 = txt.indexOf('‘‘‘'), qt2 = txt.indexOf('’’’');
	while(qt1>=0 && qt2>=0){
		if( (txt.substring(qt1-3,qt1)=='<p>' || txt.substring(qt1+3,qt1+6)=='</p') && (txt.substring(qt2+3,qt2+6)=='</p' || txt.substring(qt2-3,qt2)=='<p>') ){
			txt = txt.replace(/‘‘‘/,'<blockquote>').replace(/’’’/,'</blockquote>');
		} else {
			txt = txt.replace(/‘‘‘/,'<q>').replace(/’’’/,'</q>');
		}
		qt1 = txt.indexOf('‘‘‘'), qt2 = txt.indexOf('’’’');
	}
	txt = txt.replace(/<p><blockquote>/g,'<blockquote><p>').replace(/<blockquote><\/p>/g,'</p><blockquote>');
	txt = txt.replace(/<p><\/blockquote>/g,'</blockquote><p>').replace(/<\/blockquote><\/p>/g,'</p></blockquote>');
	return txt;
}
function 分段單兒(txt){// 僅單層
	var 諸單兒 = txt.match(/〚[列集]］.+?［[列集]〛/g);
	if(諸單兒===null){return txt;}
	for(let 單兒 of 諸單兒){
		單兒 = 單兒.replace(/<p(?=[\s>])/g,'<li').replace(/<\/p>/g,'</li>');
		txt = txt.replace(/〚[列集]］.+?［[列集]〛/,單兒);
	}
	txt = txt.replace(/(〚[列集]］)<br>/g,'$1').replace(/<br>(?=［[列集]〛)/g,'');
	txt = txt.replace(/<p>〚列］/g,'<ol class="單兒 分段單兒"><li>');
	txt = txt.replace(/〚列］<\/p>/g,'</p><ol class="單兒 分段單兒">');
	txt = txt.replace(/［列〛<\/p>/g,'</li></ol>');
	txt = txt.replace(/<p>［列〛/g,'</ol><p>');
	txt = txt.replace(/〚列］(.+?)［列〛}/g,'<ol class="單兒 分段單兒">$1</ol>');
	txt = txt.replace(/<p>〚集］/g,'<ul class="單兒 分段單兒"><li>');
	txt = txt.replace(/〚集］<\/p>/g,'</p><ul class="單兒 分段單兒">');
	txt = txt.replace(/［集〛<\/p>/g,'</li></ul>');
	txt = txt.replace(/<p>［集〛/g,'</ul><p>');
	txt = txt.replace(/〚集］(.+?)［集〛}/g,'<ul class="單兒 分段單兒">$1</ul>');
	txt = txt.replace(/<li><\/li>/g,'');
	return txt;
}
function 設置插圖(圖框, 設置){
	var cssLink = document.createElement("link"), 設置項;
	cssLink.href = "mapFrame.css";cssLink.rel = "stylesheet";cssLink.type = "text/css";
	圖框.contentWindow.document.head.appendChild(cssLink);
	if(設置.hasOwnProperty("央")){
		設置項 = 設置["央"];
		圖框.contentWindow.map.setCenter([設置項["經度"],設置項["緯度"]]);
	}
	if(設置.hasOwnProperty("縮放級")){
		圖框.contentWindow.map.setZoom(設置["縮放級"]);
	}
	if(設置.hasOwnProperty("今地名濾鏡")){
		for(let 層名 in 圖框.contentWindow.地名圖層){
			圖框.contentWindow.地名圖層[層名]["options"].cssFilter = 設置["今地名濾鏡"];
		}
	}
}
function 插圖(txt){
	var 圖起 = txt.indexOf('〚圖］'), 圖訖 = txt.indexOf('［圖〛');//
	var 諸圖 = txt.match(/〚圖］(.+?)［圖〛/g), ti = 0;
	if(圖起<0){
		return txt;
	}
	const 設置檔 = fetch('圖之設置.json');
	設置檔.then((res)=>{return res.json();}).then((圖之設置)=>{setTimeout(()=>{
		var 該圖;
		for(const 各圖名 in 圖之設置){
			該圖 = document.getElementById(各圖名);
			if(該圖!==null){
				if(該圖.contentWindow.map){
					設置插圖(該圖, 圖之設置[各圖名]);
				} else {
					該圖.addEventListener('load',function(){
						設置插圖(this, 圖之設置[各圖名]);
					});
				}
			}
		}
	}, 1000);});
	while(圖起>=0 && 圖訖>=0){
		if( (txt.substring(圖起-3,圖起)=='<p>' || txt.substring(圖起+3,圖起+6)=='</p') && (txt.substring(圖訖+3,圖訖+6)=='</p' || txt.substring(圖訖-3,圖訖)=='<p>') ){
			諸圖[ti] = 諸圖[ti].replace('〚圖］','<figure>').replace('［圖〛','</figure>');
		} else {
			諸圖[ti] = 諸圖[ti].replace('〚圖］','<fig-il>').replace('［圖〛','</fig-il>');
		}
		諸圖[ti] = 諸圖[ti].replace(/〚圖片］(.+?)［圖片〛/g,'<img src="$1">');
		諸圖[ti] = 諸圖[ti].replace(/〚地圖〛(.+?)(?=〚|<\/figure>|<\/fig-il>)/g,'<iframe src="/地圖/" id="$1" alt="$1"></iframe>');
		txt = txt.replace(/〚圖］(.+?)［圖〛/, 諸圖[ti]);
		ti++;
		圖起 = txt.indexOf('〚圖］'); 圖訖 = txt.indexOf('［圖〛');
	}
	txt = txt.replace(/<p>(?=<figure)/g,'').replace(/<figure><\/p>/g,'</p><figure>');
	txt = txt.replace(/<p><\/figure>/g,'</figure><p>').replace(/<\/figure>(?=<\/p>)/g,'</figure>');
	return txt;
}
function 旁字(txt){
	var sides = txt.match(/\{\{.+?\}\}/g);
	if(sides===null){return txt;}
	for (let r=0; r<sides.length; r++){
		sides[r] = sides[r].replace(/^\{\{/,'').replace(/\}\}$/,'');
		if(sides[r].indexOf('⏷')>=0){
			sides[r] = sides[r].replace(/^(.+?)⏷\^¯(.+?)¯\^⏶$/g,'<ruby class="mscScr">$1<span class="rightRT">$2</span></ruby>');
			sides[r] = sides[r].replace(/^(.+?)⏷\^_(.+?)_\^⏶$/g,'<ruby class="mscScr">$1<span class="leftRT">$2</span></ruby>');// 火狐rt不能自動換行.
		} else if(sides[r].indexOf('(')>=0){
			//
		}
		txt = txt.replace(/\{\{.+?\}\}/,sides[r]);
	}
	return txt;
}
function 提取旁句(txt){
	var sides = txt.match(/\(\?¯.+?¯¿\)/g);
	if(sides===null){return txt;}
	for (let r=0; r<sides.length; r++){
		sides[r] = sides[r].replace(/^\(\?¯〚錯〛/,'<small class="rubyStc rubyGap"><span><span>').replace(/¯¿\)$/,'</span></span></small>');
		sides[r] = sides[r].replace(/^\(\?¯/,'<small class="rubyStc"><span><span>').replace(/¯¿\)$/,'</span></span></small>');
		txt = txt.replace(/\(\?¯.+?¯¿\)/,sides[r]);
	}
	return txt;
}
function 寫旁句(){
	const 字號比例 = 0.5;
	var 容器 = document.getElementsByClassName("rubyStc"), 壬 = 0, 首行縮進 = 0, 襯器, 廓器;
	for (let i=0;i<容器.length;i++){
		壬 = parseFloat(window.getComputedStyle(容器[i].parentNode).getPropertyValue("margin-top"))+parseFloat(window.getComputedStyle(容器[i].parentNode).getPropertyValue("padding-top"));
		壬 += parseFloat(window.getComputedStyle(容器[i].parentNode.parentNode).getPropertyValue("margin-top"))+parseFloat(window.getComputedStyle(容器[i].parentNode.parentNode).getPropertyValue("padding-top"));
		廓器 = 容器[i].parentNode.parentNode.parentNode;
		if(!廓器.contains(document.getElementById("inMain"))){
			壬 += parseFloat(window.getComputedStyle(廓器).getPropertyValue("margin-top"))+parseFloat(window.getComputedStyle(廓器).getPropertyValue("padding-top"));
		}
		壬=Math.round(壬/正文字號);
		首行縮進 = 容器[i].offsetTop/正文字號/字號比例;
		襯器 = 容器[i].firstElementChild;
		襯器.style.top = String(-首行縮進+壬/字號比例)+"em";
		襯器.style.textIndent = String(首行縮進-壬/字號比例)+"em";
		襯器.style.height = String((每行格數-壬)/字號比例)+"em";
	}
}
function 分段(txt){
	txt = txt.replace(/\r\n/g,'\n').replace(/\r/g,'\n');
	const 連續換行 = txt.match(/\n{2,}/g);var 空行 = '';
	if (連續換行!=null){
		for (let n=0; n<連續換行.length; n++){
			if(連續換行[n].length>2){
				空行 = '<div class="空行"><br>'.repeat(連續換行[n].length-2)+'</div>';
			} else {
				空行 = '';
			}
			txt = txt.replace(/\n{2,}/,'</p>'+ 空行 +'<p>');
		}
	}
	txt = '<p>'+txt.replace(/\n/g,'<br>')+'</p>';
	return txt;
}
function 兼行(txt){
  const 所共字 = txt.match(/〚([歬前後]跨)\d+行］.+?［\1〛/g);//(/〚兼行\d+至\d+］.+?［兼行〛/g);
  if(所共字===null){return txt;}
  var 迻行數 = 0;
  for(const 各所共字 of 所共字){
    迻行數 = String(Number(各所共字.match(/\d+/)[0])/2);
    txt = txt.replace(/〚[歬前後]跨\d+行］/,'<span class="spanRows" style="'+(各所共字.indexOf('〚後跨')==0 ? 'right' : 'left')+':calc(var(--行寬)*'+迻行數+')">').replace(/［[歬前後]跨〛/,'</span>');
  }
  return txt;
}
function 特殊段(txt){// 并入渲染段.
	var 簡單戠 = ['綱','目','按','案','琵琶'];
	for(const 戠 of 簡單戠){
		txt = txt.replace(new RegExp('<p>〚'+戠+'〛','g'),'<p class="'+戠+'">');
	}
	txt = txt.replace(/<p>\*　{6,}(.+?)<\/p>/g,'<h6>$1</h6>');
	txt = txt.replace(/<p>\*　{5}(.+?)<\/p>/g,'<h5>$1</h5>');
	txt = txt.replace(/<p>\*　{4}(.+?)<\/p>/g,'<h4>$1</h4>');
	txt = txt.replace(/<p>\*　{3}(.+?)<\/p>/g,'<h3>$1</h3>');
	txt = txt.replace(/<p>\*　{2}(.+?)<\/p>/g,'<h2>$1</h2>');
	txt = txt.replace(/<p>\*　(.+?)<\/p>/g,'<h1>$1</h1>');
	txt = txt.replace(/〚題下〛(.+?)(?=<\/h\d>)/g,'<span class="te2hra4">$1</span>');
	if(調距["敘"]==0){
		txt = txt.replace(/<p>([敘序]:|〚[敘序]〛)/g,'<p class="sio4">');
	} else {
		txt = txt.replace(/<p>([敘序]:|〚[敘序]〛)/g,'<p class="sio4" style="letter-spacing:'+String(調距["敘"])+'em">');
	}
	if(調距["跋"]==0){
		txt = txt.replace(/<p>(跋:|〚跋〛)/g,'<p class="buad8">');
	} else {
		txt = txt.replace(/<p>(跋:|〚跋〛)/g,'<p class="buad8" style="letter-spacing:'+String(調距["跋"])+'em">');
	}
	txt = txt.replace(/<p>〚[注註]〛/g,'<p class="註">');
	var mc = txt.match(/<p>〚目次〛(.+?)<\/p>/);
	if (mc!==null){
		mc[0] = mc[0].replace(/<p>〚目次〛(<br>){0,1}/,'<ol class="卷內目次"><li><a onclick="滾到節(1)">');
		let mi = 0;
		while(mc[0].indexOf('<br>')>=0){
			mc[0] = mc[0].replace(/<br>/,'</a></li><li><a onclick="滾到節('+String(mi+2)+')">');
			mi++;
		}
		while(mc[0].indexOf('			')>=0){
			mc[0] = mc[0].replace(/\t{3,}/,'</a></li><li><a onclick="滾到節('+String(mi+2)+')">');
			mi++;
		}
		/*if(mc[0].replace(/<p>〚目次〛<br>/,'').indexOf('<br>')>=0){
			mc[0] = mc[0].replace(/<br>/g,鏈目);
		} else {
			mc[0] = mc[0].replace(/\t{3,}/g,鏈目);
		}*/
		mc[0] = mc[0].replace(/<\/p>/,'</a></li></ol>');
		txt = txt.replace(/<p>〚目次〛(.+?)<\/p>/,mc[0]);
	}
	var 諸單兒 = txt.match(/<p>〚[列集]〛(.+?)<\/p>/g);
	if (諸單兒 !== null){
		let 單兒籤 = '';
		for (let 單兒 of 諸單兒){
			單兒籤 = (單兒.indexOf('<p>〚列〛')==0) ? 'ol' : 'ul';
			單兒 = 單兒.replace(/<p>〚[列集]〛(<br>){0,1}/, '<'+ 單兒籤 +' class="單兒 段單兒"><li>');
			單兒 = 單兒.replace(/<br>/g,'</li><li>').replace(/\t{2,}/g,'</li><li>');
			單兒 = 單兒.replace(/<\/p>/,'</li></'+ 單兒籤 +'>');
			txt = txt.replace(/<p>〚[列集]〛(.+?)<\/p>/, 單兒);
		}
	}
	txt = txt.replace(/<p>〚([譔撰作校]者|落款)〛(.+?)<\/p>/g,'<div class="$1" style="margin-bottom:'+String(行尾空白長)+'em">$2</div>');

	txt = txt.replace(/<p>\s*?<\/p>/g,'');
	return txt;
}
function 縱中橫碼(num){
	var s = '', sgn = '', L = 1, 大數 = '', 小數 = '';
	if (String(num).slice(0,1)=='-' || String(num).slice(0,1)=='−'){sgn = '＼';}
	s = String(num).replace(/[-−+]/g,'').split('.');
	if (s.length>2) {return num;}
	if (s[0].length==0) {s[0] = '0';}
	L = s[0].length;// 大數位數 
	var c = L % 4;
	if (c==3 || c==2 || c==1) {
		大數 = '<span class="lrtb o'+ c.toString()+'d">'+ s[0].slice(0,c) + '</span>';
	}
	for (c=L%4; c<L; c+=4) {
		大數 += '<span class="lrtb o4d">'+ s[0].substr(c,4) +'</span>';
	}
	if (s[1] === undefined) {
		return (sgn + 大數);
	}
	L = s[1].length;// 小數位數 
	for (c=0; c<L-L%4; c+=4) {
		小數 += '<span class="lrtb o4d">'+ s[1].substr(c,4) +'</span>';
	}
	if (L%4==3 || L%4==2 || L%4==1) {
		小數 += '<span class="lrtb o'+(L%4).toString()+'d">'+ s[1].slice(c) +'</span>';
	}
	return (sgn + 大數 +'又'+ 小數);
}
function 數碼(txt){
	const xs = txt.match(/〚筭］.+?［筭〛/g) || [];
	for(let x of xs){
		x = x.replace(/^〚筭］|［筭〛$/g,'');
		x = x.replace(/[-−+]{0,1}\d+\.{0,1}\d*/g, 縱中橫碼);
		x = x.replace(/>:</g,'><span class="midColon">:</span><');
		x = x.replace(/＼*[〇〡-〩]+/g, (m)=>{
			let d = 整數籌(m);
			return '<data class="cnumDr '+ d["類"]+'" style="font-stretch:'+ String(d["寬窄"])+'%">'+(d["負號"]?'<span class="DrNegSgn">'+d["負號"]+'</span>':'')+ d["數"]+'</data>'});
		txt = txt.replace(/〚筭］.+?［筭〛/, x);
	}
	txt = txt.replace(/〚碼子］(.+?)［碼子〛/g, '<data class="cnumMZ">$1</data>');
	txt = txt.replace(/〚數卦］(.+?)［數卦〛/g, '<data class="divine3">$1</data>');
	return txt;
}
function 字樣(txt,圖夾='圖'){
	const 格式 = ['.svg','.png'];
	const s = txt.matchAll(/〚字樣］(.+?)［字樣〛/g);
	var 字 = '', 圖徑;
	圖夾 = 圖夾.replace(/\\/g,'/');
	if(圖夾.slice(-1)!='/'){圖夾 += '/';}
	for(const 樣 of s){
		if(樣[1].indexOf('◎')<0){
			字 = 樣[1];
			圖徑 = [圖夾 + 字, 圖夾 + 字.codePointAt(0).toString(16), 圖夾 + 字.codePointAt(0).toString(16).toUpperCase()];
			txt = txt.replace(/〚字樣］.+?［字樣〛/,'<dfn class="字樣" data-char="'+字+'"><img src="'+ 圖徑[0]+格式[0] +'" onerror="this.onerror=null; this.src=\''+圖徑[0]+格式[1]+'\'" /><b>'+ 字 +'</b></dfn>');
		} else {
			[字, 圖徑] = 樣[1].split('◎');
			圖徑 = 圖夾 + 圖徑;
			txt = txt.replace(/〚字樣］.+?［字樣〛/,'<dfn class="字樣" data-char="'+字+'"><img src="'+ 圖徑 +'" /><b>'+ 字 +'</b></dfn>');
			// srcset="'+ 圖徑.join('.svg, ')+'.svg, '+ 圖徑.join('.png, ')+'.png"
		}
	}
	return txt;
}
function toHTML(cvTXT){
  let x = cvTXT;//document.getElementById(cvID).innerHTML;
	x = 數碼(x);
  x = 畫表格(x);
  x = x.replace(/\(\(\-(.+?)\-\)\)/g,'<small class="dhgd">$1</small>');
  x = x.replace(/᚜\[¯(.+?)¯\]᚛/g,'<sup class="rightLine">$1</sup>');
  x = x.replace(/᚜\[_(.+?)_\]᚛/g,'<sub class="leftLine">$1</sub>');
  x = x.replace(/᚜\(¯(.+?)¯\)᚛/g,'<small class="rightLCmt">$1</small>');
  x = x.replace(/᚜\(_(.+?)_\)᚛/g,'<small class="leftLCmt">$1</small>');
  //x = x.replace(/᚜\[=(.+?)¯\|_(.+?)=\]᚛/g,'<span class="twoLine">$1<br>$2</span>');
  x = x.replace(/᚜\(=(.+?)¯\|_(.+?)=\)᚛/g,'<small class="twoLCmt">$1<br>$2</small>');
   x = x.replace(/〔〔\-(.+?)\-〕〕/g,'<dt>$1</dt>');
  //var titleLv = x.match(/^　+/)[0].length;
  /*x = x.replace(/^([^　].+)$/mg,'<\/div>$1</p>');*/
  x = 分段(x);
  //x = '<p>'+x.replace(/\r\n|\n\r|\r|\n/g,'</p><p>')+'</p>';
  //x = x.replace(/<\/p><p><\/p><p>/g,'</p><br><p>');
  x = 分節(x);
  x = 引詩文(x);
  x = 分段單兒(x);
  x = 插圖(x);
  x = 尾注(x);
  x = x.replace(/<p><table/g,'<table');x = x.replace(/<\/table><\/p>/g,'</table>');
  x = 特殊段(x);
  x = 渲染段(x);
  x = 兼行(x);
  if(調距["正文"]!=0){
    x = x.replace(/<p( class=".*?"){0,1}>/g,'<p$1 style="letter-spacing:'+String(調距["正文"])+'em">');
  }
  //x = x.replace(/ ␤ /g,'<br>');
  x = 旁字(x);
  x = 提取旁句(x);
  x = 字樣(x);
  //x = x.replace(/<p([^>]+?)style="(.+?)">\s*?\t⇥/g,'<p$1 style="$2; text-align:right; margin-bottom:'+String(行尾空白長)+'em">');
  //x = x.replace(/>\s*?\t⇥/g,' style="text-align:right; margin-bottom:'+String(行尾空白長)+'em">');
  x = x.replace(/\\([﹅﹆•◦●○◒◓◉◎▲△])\{(.+?)\}/g,'<span style="-webkit-text-emphasis-style:\'$1\';text-emphasis-style:\'$1\'">$2</span>');
  x = x.replace(/\\([﹅﹆•◦●○◒◓◉◎▲△])\[(.*?)\]\{(.+?)\}/g,'<span style="-webkit-text-emphasis-style:\'$1\';text-emphasis-style:\'$1\';-webkit-text-emphasis-color:var(--$2);text-emphasis-color:var(--$2)">$3</span>');
  //x = x.replace(/\\([﹅﹆•◦●○◉◎▲△])\{(.+?)\}/g,'<em data-emsym="$1">$2</em>');
  //x = x.replace(/\\([﹅﹆•◦●○◉◎▲△])\[(.*?)\]\{(.+?)\}/g,'<em data-emsym="$1" style="-webkit-text-emphasis-color:var(--$2);text-emphasis-color:var(--$2)">$3</em>');
  //x = x.replace(/\\圖\[(.*?)\]\[高(\d+.+?)\]\{(.+?)\}/g,'<img alt="$1" style="height:$2" src="$3" />');
  x = x.replace(/\[([﹅﹆•◦●○◒◓◉◎▲△■◆])\]/g,'<span class="ryme" data-韵標="$1"></span>');
  x = x.replace(/\[(\d)([﹅﹆•◦●○◒◓◉◎▲△■◆])\]/g,'<span class="ryme" data-韵標="$2" style="color:var(--韵抹$1) !important"></span>');
  x = x.replace(/\[(\d+?)(\d)([﹅﹆•◦●○◒◓◉◎▲△■◆])\]/g,'<span class="ryme" data-韵標="$3" style="color:var(--韵抹$2) !important"></span>');
  x = x.replace(/〚單擡〛/g,'<b class="headup1"></b>').replace(/〚雙擡〛/g,'<b class="headup2"></b>').replace(/〚三擡〛/g,'<b class="headup3"></b>');
  x = x.replace(/<p[^<>]*?><\/p>/g,'');
  return x;//document.getElementById(cvID).innerHTML = x;
}
function 陰文和墨圍(x){
	x = x.replace(/\[【(.+?)】\]/g,'<i class="hideCopy">[</i><cite class="imVang">$1</cite><i class="hideCopy">]</i>');
	x = x.replace(/\(【(.+?)】\)/g,'<i class="hideCopy">(</i><cite class="imHiuan">$1</cite><i class="hideCopy">)</i>');
	x = x.replace(/〔【(.+?)】〕/g,'<i class="hideCopy">〔</i><cite class="im6grug">$1</cite><i class="hideCopy">〕</i>');
	x = x.replace(/\!\[⬣(.+?)⬣\]¡/g,'<cite class="imLantn">$1</cite>');
	x = x.replace(/\!\[●(.+?)●\]¡/g,'<cite class="imCirc">$1</cite>');
	x = x.replace(/\[〖(.+?)〗\]/g,'<i class="hideCopy">[</i><cite class="mhVang">$1</cite><i class="hideCopy">]</i>');
	x = x.replace(/\(〖(.+?)〗\)/g,'<i class="hideCopy">(</i><cite class="mhHiuan">$1</cite><i class="hideCopy">)</i>');
	x = x.replace(/〔〖(.+?)〗〕/g,'<i class="hideCopy">〔</i><cite class="mh6grug">$1</cite><i class="hideCopy">〕</i>');
	x = x.replace(/\!\[⎔(.+?)⎔\]¡/g,'<cite class="mhLantn">$1</cite>');
	x = x.replace(/\!\[○(.+?)○\]¡/g,'<cite class="mhCirc">$1</cite>');
	x = x.replace(/﹙〖(.+?)〗﹚/g,'<i class="hideCopy">﹙</i><cite class="mhHiuanNR">$1</cite><i class="hideCopy">﹚</i>');
	x = x.replace(/﹝〖(.+?)〗﹞/g,'<i class="hideCopy">﹝</i><cite class="mh8grugNR"><span>$1</span></cite><i class="hideCopy">﹞</i>');
	return x;
}
function 圈點抹(x){
	x = x.replace(/\?\?•(.+?)•¿¿/g,'<mark class="colateOF">$1</mark>');
	x = x.replace(/\?\?◦(.+?)◦¿¿/g,'<mark class="colateOO">$1</mark>');
	x = x.replace(/\?\?\-(.+?)\-¿¿/g,'<mark class="colateMS">$1</mark>');
	x = x.replace(/\?\?(.+?)¿¿/g,'<mark class="colateTC">$1</mark>');
	x = x.replace(/\!\!¯•(.+?)•¯¡¡/g,'<strong class="strROF">$1</strong>');
	x = x.replace(/\!\!¯◦(.+?)◦¯¡¡/g,'<strong class="strROO">$1</strong>');
	function 旁點(txt) {
		var sentens = [...txt.matchAll(/\!\!¯([﹆﹅]).+?\1¯¡¡/g)], tgs;
		if (sentens.length>0) {
			txt = txt.replace(/\!\!¯([﹆﹅]).+?\1¯¡¡/g,'｛$1｝');
			for (let i = 0; i < sentens.length; i++) {
				sentens[i][0] = sentens[i][0].replace(/([〇\u3400-\u9FFF])/g,'$1¯¯'+sentens[i][1]);//sentens[i] = sentens[i].split("").join("﹆");
				sentens[i][0] = sentens[i][0].replace(/([\u{20000}-\u{2A6DF}\u{2A700}-\u{2CEAF}])/gu,'$1¯¯'+sentens[i][1]);
				//sentens[i] = sentens[i].replace(/韵﹆標﹆/,'韵標').replace(/韵﹆抹﹆/,'韵抹');
				tgs = sentens[i][0].match(/<[^\/].+?>/g);
				if (tgs !== null) {
					sentens[i][0] = sentens[i][0].replace(/<[^\/].+?>/g,'｛〈〉｝');
					for (let j = 0; j < tgs.length; j++) {
						tgs[j] = tgs[j].replace(/([〇\u3400-\u9FFF])¯¯[﹆﹅]/g,'$1');
						sentens[i][0] = sentens[i][0].replace(/｛〈〉｝/,tgs[j]);
					}
				}
				txt = txt.replace(/｛[﹆﹅]｝/,sentens[i][0].slice(4,-4));
			}
		}
		return txt;
	}
	x = 旁點(x);
	return x;
}
function 點行間(elID,clr){
	var x = document.getElementById(elID).innerHTML;
	if(clr === undefined){
		clr = 'inherit';
	} else if(!clr.match(/^#[A-Fa-f0-9]|^rgb\(/)) {
		clr = 'var(--'+ clr +')';
	}
	//x = x.replace(/([◎○□])/g,'<span style="color:'+clr+'">$1</span>');
	x = x.replace(/([^「『])([「『])(?![「『』」])/g,'$1<span class="hgbdSysQt1">$2</span>');
	x = x.replace(/([^「『』」])([』」])(?![』」])/g,'$1<span class="hgbdSysQt2">$2</span>');
	x = x.replace(/([，、。．])/g, '<span class="jmhgbd hgbdSysSm" lang="zh-Hans-CN">$1</span>');/*細明體,MingLiU,PingFang TC,Source Han Sans TC,Noto Sans CJK TC*/
	x = x.replace(/([：；])/g, '<span class="jmhgbd hgbdSysMd" lang="zh-Hans-CN">$1</span>');
	x = x.replace(/([？！])/g, '<span class="jmhgbd hgbdSysTl" lang="zh-Hans-CN">$1</span>');
	x = x.replace(/』」/g,'<span class="hgbd2Qt2"><span>』</span><span>」</span></span>');
	x = x.replace(/「『/g,'<span class="hgbd2Qt1"><span>「</span>『</span>');
	//x = x.replace(/」「/g,'<span style="letter-spacing: -1em">」</span>「');
	//x = x.replace(/』『/g,'<span style="letter-spacing: -1em">』</span>『');
	x = x.replace(/）（/g,'<span style="letter-spacing: -1em">）</span>（');
	x = 圈點抹(x);
	x = x.replace(/¯¯([﹅﹆ロ])/g,'<span class="jmhgbd hgbdRightEm" lang="zh-Hans-CN">$1</span>');
	//if (系統 == 'Android') {x = x.replace(/([〪〭〫〬])/g,'<span class="sigaugxieng">$1</span>');} else {
	x = x.replace(/(〪)/g,'<span class="sigaugxieng fkLevel lrtb">$1</span>');
	x = x.replace(/(〫)/g,'<span class="sigaugxieng fkRise lrtb">$1</span>');
	x = x.replace(/(〬)/g,'<span class="sigaugxieng fkDept">$1</span>');
	x = x.replace(/(〭)/g,'<span class="sigaugxieng fkEnter lrtb">$1</span>');
	//}// 四角聲符 
	//x = x.replace(/([〪〭〫〬])(.)/g,'$2<span class="sigrugxieng">$1</span>');
	x = 陰文和墨圍(x);
	//x = x.replace(/【(.+?)】/g,'<span style="padding:1px 4px; margin:-1px -2px; border-radius:5px; background:#000; color:#fff">$1</span>');
	x = x.replace(/〖(.+?)〗/g,'<span style="padding:0 4px; margin:-2px -2px; border-radius:5px; border:1px solid gray">$1</span>');
	x = x.replace(/\{（(.+?)）\}/g,'<del>$1</del>');
	x = x.replace(/\{［(.+?)］\}/g,'<ins>$1</ins>');
	x = x.replace(/《(.+?)》/g,'<i class="hideCopy">《</i><u class="xiomeng">$1</u><i class="hideCopy">》</i>');
	x = x.replace(/〈(.+?)〉/g,'<i class="hideCopy">〈</i><u class="xiomengRight">$1</u><i class="hideCopy">〉</i>');
	x = x.replace(/(·)/g,'<span class="midDotPrd" style="bottom: 0.5em">$1</span>');
	//x = x.replace(/[·・]》/g,'</u><u class="xiomeng">');
	x = x.replace(/‹(.+?)›/g,'<i class="hideCopy">‹</i><u class="juenmeng"><span>$1</span></u><i class="hideCopy">›</i>');
	x = x.replace(/\{(.+?)\|(.+?)\}/g,'<span class="rightRuby" data-rrt="$2">$1</span>');
	x = x.replace(/([…〻])(?!$)/g,'<span style="display:inline-block">$1</span>');
	document.getElementById(elID).innerHTML = x;
}// 正文標點.
function 行間標點(elID,clr){
  var x = document.getElementById(elID).innerHTML;
  x = x.replace(/？+！+/g,'⁈');x = x.replace(/！+？+/g,'⁉');x = x.replace(/？{2,}/g,'⁇');x = x.replace(/！{2,}/g,'‼');
  if(clr === undefined){
    x = x.replace(/([，、､。｡．：；？！⁇⁈⁉‼「」『』（）…〻々])/g,'<span class="jmhgbd0">$1</span>');
  } else {
    if(clr.match(/^#[A-Fa-f0-9]|^rgb\(/)){
      x = x.replace(/([，、､。｡．：；？！⁇⁈⁉‼「」『』（）…〻々])/g,'<span class="jmhgbd0" style="color:'+clr+'">$1</span>');
    }else{
      x = x.replace(/([，、､。｡．：；？！⁇⁈⁉‼「」『』（）…〻々])/g,'<span class="jmhgbd0" style="color:var(--'+clr+')">$1</span>');
    }
  }
  x = 圈點抹(x);
  x = x.replace(/([﹅﹆])/g,'<span class="jmhgbd0">$1</span>');
  x = x.replace(/([〪〭〫〬])/g,'<span class="sigrugxieng">$1</span>');
  x = 陰文和墨圍(x);
  //x = x.replace(/【(.+?)】/g,'<span style="padding:1px 4px; margin:-1px -2px; border-radius:5px; background:#000; color:#fff">$1</span>');
  x = x.replace(/〖(.+?)〗/g,'<span style="padding:0 4px; margin:-2px -2px; border-radius:5px; border:1px solid gray">$1</span>');
  x = x.replace(/《(.+?)》/g,'<u class="xiomeng">$1</u>');
  x = x.replace(/〈(.+?)〉/g,'<u class="xiomengRight">$1</u>');
  x = x.replace(/(·)/g,'<span class="midDotPrd" style="bottom: 0.5em; color:var(--標點色)">$1</span>');
  //x = x.replace(/[·・]》/g,'</u><u class="xiomeng">');
  x = x.replace(/‹(.+?)›/g,'<u class="juenmeng"><span>$1</span></u>');
  x = x.replace(/\{(.+?)\|(.+?)\}/g,'<span class="rightRuby" data-rrt="$2">$1</span>');
  document.getElementById(elID).innerHTML = x;
}
function 點字下(gdEltxt) {
    var y = gdEltxt;//gdEl.innerHTML;
    y = y.replace(/(^|[^﹁﹃])([﹁﹃])(?![﹁﹃﹄﹂])/g,'$1<span class="zzbdQt1">$2</span>');
    y = y.replace(/([^﹁﹃﹄﹂])([﹄﹂])(?=$|[^﹄﹂])/g,'$1<span class="zzbdQt2">$2</span>');
    if( 系統 =='Android' || 系統 =='iPhone'){
      y = y.replace(/([︐︑︒・])/g, '<span class="gdzhbd zzbdMSm">$1</span>');
      y = y.replace(/([︓︔])/g, '<span class="gdzhbd zzbdMMd">$1</span>');
    } else {
      y = y.replace(/([︐︑︒・])/g, '<span class="gdzhbd zzbdSm">$1</span>');
      y = y.replace(/([︓︔])/g, '<span class="gdzhbd zzbdMd">$1</span>');
    }
    y = y.replace(/([︖︕])/g, '<span class="gdzhbd zzbdTl">$1</span>');
    y = y.replace(/﹄﹂/g,'<span class="zzbd2Qt2"><span>﹄</span>﹂</span>');
    y = y.replace(/﹁﹃/g,'<span class="zzbd2Qt1"><span>﹁</span>﹃</span>');
    //y = y.replace(/﹂﹁/g,'<span style="position:relative; bottom:0.625em; letter-spacing: -1em; color:var(--標點色); font-family:爾雅楷體">﹂﹁</span>');
    //y = y.replace(/﹄﹃/g,'<span style="letter-spacing: -1em; font-family:爾雅楷體">﹄﹃</span>');
    y = y.replace(/︶︵/g,'<span style="letter-spacing: -1em">︶</span>︵');
    y = y.replace(/([¯ˇˋˆ])/g,'<span class="sigaugxieng">$1</span>');// 四角聲符 
    y = y.replace(/\[︻(.+?)︼\]/g,'<i class="hideCopy">[</i><cite class="imVang"><span>$1</span></cite><i class="hideCopy">]</i>');
    y = y.replace(/\(︻(.+?)︼\)/g,'<i class="hideCopy">(</i><cite class="imHiuan"><span>$1</span></cite><i class="hideCopy">)</i>');
    y = y.replace(/︹︻(.+?)︼︺/g,'<i class="hideCopy">〔</i><cite class="im6grug"><span>$1</span></cite><i class="hideCopy">〕</i>');
    y = y.replace(/\[(.+?)\]/g,'<i class="hideCopy">[</i><cite class="mhVang"><span>$1</span></cite><i class="hideCopy">]</i>');
    y = y.replace(/\((.+?)\)/g,'<i class="hideCopy">(</i><cite class="mhHiuan"><span>$1</span></cite><i class="hideCopy">)</i>');
    y = y.replace(/︹(.+?)︺/g,'<i class="hideCopy">〔</i><cite class="mh6grug"><span><span>$1</span></span></cite><i class="hideCopy">〕</i>');
    y = y.replace(/\[︻(.+?)$/g,'<cite class="imVangSt">$1</cite>');
    y = y.replace(/\(︻(.+?)$/g,'<cite class="imHiuanSt">$1</cite>');
    y = y.replace(/︹︻(.+?)$/g,'<cite class="im6grugSt">$1</cite>');
    y = y.replace(/\[(.+?)$/g,'<cite class="mhVangSt"><span>$1</span></cite>');
    y = y.replace(/\((.+?)$/g,'<cite class="mhHiuanSt">$1</cite>');
    y = y.replace(/︹(.+?)$/g,'<cite class="mh6grugSt">$1</cite>');
    y = y.replace(/^(.+?)︼\]/g,'<cite class="imVangEd"><span>$1</span></cite>');
    y = y.replace(/^(.+?)︼\)/g,'<cite class="imHiuanEd"><span>$1</span></cite>');
    y = y.replace(/^(.+?)︼︺/g,'<cite class="im6grugEd"><span>$1</span></cite>');
    y = y.replace(/^(.+?)\]/g,'<cite class="mhVangEd"><span>$1</span></cite>');
    y = y.replace(/^(.+?)\)/g,'<cite class="mhHiuanEd"><span>$1</span></cite>');
    y = y.replace(/^(.+?)︺/g,'<cite class="mh6grugEd"><span>$1</span></cite>');
    y = y.replace(/︻(.+?)︼/g,'<span class="imHiuan"><span>$1</span></span>')//'<span style="padding:0.1em 0 0 0; margin:-0.1em 0 0 0; border-radius:5px; background:#000; color:#fff">$1</span>');
    y = y.replace(/(.+?)/g,'<span style="padding:0 4px; margin:-2px -2px; border-radius:5px; border:1px solid gray">$1</span>');
    //y = y.replace(/︽(.+?)[·・](.+?)︾/g,'<u class="xiomeng">$1</u><u class="xiomeng">$2</u>');y = y.replace(/︿(.+?)[·・](.+?)﹀/g,'<u class="xiomeng">$1</u><u class="xiomeng">$2</u>');
    y = y.replace(/\{︵(.+?)︶\}/g,'<del>$1</del>');
    y = y.replace(/\{﹇(.+?)﹈\}/g,'<ins>$1</ins>');
    y = y.replace(/︽(.+?)︾/g,'<i class="hideCopy">《</i><u class="xiomeng"><span>$1</span></u><i class="hideCopy">》</i>');y = y.replace(/︿(.+?)﹀/g,'<span style="color:purple">$1</span>');
    y = y.replace(/︽([^︾]+)$/g,'<u class="xiomengSt"><span>$1</span></u>');y = y.replace(/^([^︾]+)︾/g,'<u class="xiomengEd"><span>$1</span></u>');
    //y = y.replace(/︿([^﹀]+)$/g,'<u class="xiomeng">$1</u>');y = y.replace(/^([^﹀]+)﹀/g,'<u class="xiomeng">$1</u>');
    //y = y.replace(/[·・]》/g,'</u><u class="xiomeng">');
    y = y.replace(/(･)/g,'<span class="midDotPrd" style="bottom: 0.625em">$1</span>');
    //y = y.replace(/︳(.+?)︳/g,'<u class="juenmeng"><span>$1</span></u>');
    //y = y.replace(/︳([^︳]+)$/g,'<u class="juenmengSt"><span>$1</span></u>');y = y.replace(/^([^︳]+)︳/g,'<u class="juenmengEd"><span>$1</span></u>');
    y = y.replace(/﹤(.+?)﹥/g,'<i class="hideCopy">‹</i><u class="juenmeng"><span>$1</span></u><i class="hideCopy">›</i>');
    y = y.replace(/﹤([^﹥]+)$/g,'<u class="juenmengSt"><span>$1</span></u>');y = y.replace(/^([^﹥]+)﹥/g,'<u class="juenmengEd"><span>$1</span></u>');
    if(調距["注中小字"]==-1){
      y = y.replace(/﹇﹇(.+?)﹈﹈/g,'<small class="DRinDRhf">$1</small>');
      y = y.replace(/﹇﹇([^﹈]+)$/g,'<small class="DRinDRhf">$1</small>');y = y.replace(/^(.+?)﹈﹈/g,'<small class="DRinDRhf">$1</small>');
    } else {
      y = y.replace(/﹇﹇(.+?)﹈﹈/g,'<small class="DRinDRhf" style="letter-spacing:'+String(調距["注中小字"]+2)+'em">$1</small>');
      y = y.replace(/﹇﹇([^﹈]+)$/g,'<small class="DRinDRhf" style="letter-spacing:'+String(調距["注中小字"]+2)+'em">$1</small>');y = y.replace(/^(.+?)﹈﹈/g,'<small class="DRinDRhf" style="letter-spacing:'+String(調距["注中小字"]+2)+'em">$1</small>');
    }
    if(調距["注中注"]==0){
      y = y.replace(/﹇(.+?)﹈/g,'<span class="DRinDR">$1</span>');
      y = y.replace(/﹇([^﹈]+)$/g,'<span class="DRinDR">$1</span>');y = y.replace(/^(.+?)﹈/g,'<span class="DRinDR">$1</span>');
    } else {
      y = y.replace(/﹇(.+?)﹈/g,'<span class="DRinDR" style="letter-spacing:'+String(調距["注中注"]+0.25)+'em">$1</span>');
      y = y.replace(/﹇([^﹈]+)$/g,'<span class="DRinDR" style="letter-spacing:'+String(調距["注中注"]+0.25)+'em">$1</span>');y = y.replace(/^(.+?)﹈/g,'<span class="DRinDR" style="letter-spacing:'+String(調距["注中注"]+0.25)+'em">$1</span>');
    }
    y = y.replace(/([　︙〻])(?!$)/g,'<span style="display:inline-block">$1</span>');// 空格(挪擡)等居行首.
    return y;//gdEl.innerHTML = y;
}// 夾注標點.
function 字格數(文){//未用
	var 乙 = 文.replace(/[︐︑︒︓︔︖︕︽︾︿﹀･・﹁﹂﹃﹄︻︼︵︶﹇﹈¯ˇˋˆ︳﹤﹥\(\)\[\]\{\}︹︺〘〙]/g,'');
	乙 = 乙.replace(/<small class="(pCmtN|gCmtN).+?<\/small>/g,'').replace(/<.+?>/g,'');
	const latins = 乙.replace(/<.+?>/g,'').match(/[A-Z/a-z.,:;?! '\-*+"À-ÿĀ-ſ]/g);
	var 丙 = 乙.length;
	if (latins!==null){丙 -= latins.length;}
	return 丙;
}
function 雙行夾注(domid){var 甲;if(domid===undefined){甲=document.getElementsByClassName("grebdriuo")}else{甲=document.getElementById(domid).getElementsByClassName("grebdriuo")}var 天=new Array(甲.length);var 乙="",丙=0,丁=0,戊=0;var 己,f=new Array(3);var 庚='',辛=2;var 壬=0;var 癸=1,焉=1,哉=2,乎=0;var 也="",於="";var 裡="";for(var i=0;i<甲.length;i++){/*if((系統=="Android"||系統=="iPhone"||系統=="iPad")&&瀏覽器.名!="火狐"&&甲[i].parentNode.tagName!="LI"){甲[i].style.display="none";continue}*/天[i]=甲[i].innerHTML;乙=天[i].replace(/[︐︑︒︓︔︖︕︽︾︿﹀･・﹁﹂﹃﹄︻︼︵︶﹇﹈¯ˇˋˆ︳﹤﹥\(\)\[\]\{\}︹︺〘〙]/g,'');乙=乙.replace(/<small class="(pCmtN|gCmtN).+?<\/small>/g,'').replace(/<.+?>/g,'');丙=乙.length;if(丙<=2){甲[i].className="grebdriuo ig_gd";甲[i].innerHTML=點字下(天[i]);continue}己=document.createElement("span");己.className="beforegd";甲[i].parentNode.insertBefore(己,甲[i]);丁=Math.round(己.offsetTop/正文字號);if(丙<=(每行格數-(丁>=每行格數?0:丁)+0)*2){甲[i].className="grebdriuo sht_gd";甲[i].style.height=(1.25*Math.ceil(丙/2)+0.125)+"em";甲[i].innerHTML=點字下(天[i])}else{if(丁==0){丁=每行格數}壬=parseFloat(window.getComputedStyle(甲[i].parentNode).getPropertyValue("margin-top"))+parseFloat(window.getComputedStyle(甲[i].parentNode).getPropertyValue("padding-top"));壬=Math.round(壬/正文字號);辛=Math.ceil(丙/2)-(每行格數-丁);癸=(辛)/(每行格數-壬);焉=Math.floor(癸);哉=Math.ceil(癸)+(每行格數<丁?0:1);乎=辛-焉*(每行格數-壬);if(乎==0){乎=每行格數-壬}庚=String((丁-壬)*1.25)+"em";也="polygon(0 "+String(乎*1.25+0.15)+"em,calc(var(--跨註行寬)*1.9) "+String(乎*1.25+0.15)+"em,calc(var(--跨註行寬)*1.9) 100%,0 100%)";於="polygon(0 0,0 "+String(乎*1.25+0.1)+"em,calc(var(--跨註行寬)*2) "+String(乎*1.25+0.1)+"em,calc(var(--跨註行寬)*2) 100%,calc(var(--跨註行寬)*"+String(哉*2)+") 100%,calc(var(--跨註行寬)*"+String(哉*2)+") "+庚+",calc(var(--跨註行寬)*"+String(哉*2-2)+") "+庚+",calc(var(--跨註行寬)*"+String(哉*2-2)+") 0)";甲[i].className="grebdriuo long_gd";甲[i].style.width="calc(var(--跨註行寬)*"+String(Math.ceil(哉)*2)+")";甲[i].style.shapeOutside=甲[i].style.clipPath=於;裡='<span class="gdHeadflt" style="height:'+庚+'"></span><span class="gdTailflt" style="clip-path:'+也+";shape-outside:"+也+'"></span>'+點字下(天[i]);甲[i].innerHTML="<span>"+裡+"</span>";if(甲[i].nextSibling!==null){己=document.createElement("span");己.className="sprAftgd";己.style.height=String(乎)+"em";甲[i].insertAdjacentElement('afterend',己)}}甲[i].dataset.jdgs=Math.ceil(丙/2)}};
function 調整雙行夾注(domid="miun"){
	var 器 = document.getElementById(domid);
	const 諸夾註歬 = 器.getElementsByClassName("beforegd");
	var 丁 = 0, 佔大格數 = 2;
	for(const 夾註歬 of 諸夾註歬){
		let 夾註 = 夾註歬.nextElementSibling;
		佔大格數 = Number(夾註.dataset.jdgs);
		丁 = Math.round(夾註歬.offsetTop/正文字號);
		if(佔大格數<=每行格數-(丁>=每行格數?0:丁) && 夾註.classList.contains("long_gd")){// 不跨大行
			夾註.classList.replace("long_gd","sht_gd");
			let 字距 = 夾註.style.letterSpacing;夾註.style.width = 夾註.style.clipPath = 夾註.style.shapeOutside = "";
			夾註.style.height = (1.25*佔大格數+0.125).toString()+"em";夾註.style.letterSpacing = 字距;
			夾註.firstChild.removeChild(夾註.getElementsByClassName("gdHeadflt")[0]);
			夾註.firstChild.removeChild(夾註.getElementsByClassName("gdTailflt")[0]);
			夾註.innerHTML = 夾註.innerHTML.replace(/^<span>|<\/span>$/g,'');
			if(夾註.nextElementSibling && 夾註.nextElementSibling.className=="sprAftgd"){
				夾註.nextElementSibling.remove();
			}
		} else if(佔大格數>每行格數-(丁>=每行格數?0:丁)){
			if(丁==0){丁 = 每行格數;}
			let 壬 = parseFloat(window.getComputedStyle(夾註.parentNode).getPropertyValue("margin-top"))+parseFloat(window.getComputedStyle(夾註.parentNode).getPropertyValue("padding-top"));
			壬 = Math.round(壬/正文字號);
			let 辛 = 佔大格數-(每行格數-丁);
			let 癸=(辛)/(每行格數-壬);
			let 焉 = Math.floor(癸), 哉 = Math.ceil(癸)+(每行格數<丁?0:1);
			let 乎 = 辛-焉*(每行格數-壬);
			if(乎==0){乎=每行格數-壬;}
			let 庚 = String((丁-壬)*1.25)+"em";
			let 也="polygon(0 "+String(乎*1.25+0.15)+"em,calc(var(--跨註行寬)*1.9) "+String(乎*1.25+0.15)+"em,calc(var(--跨註行寬)*1.9) 100%,0 100%)";
			let 於="polygon(0 0,0 "+String(乎*1.25+0.1)+"em,calc(var(--跨註行寬)*2) "+String(乎*1.25+0.1)+"em,calc(var(--跨註行寬)*2) 100%,calc(var(--跨註行寬)*"+String(哉*2)+") 100%,calc(var(--跨註行寬)*"+String(哉*2)+") "+庚+",calc(var(--跨註行寬)*"+String(哉*2-2)+") "+庚+",calc(var(--跨註行寬)*"+String(哉*2-2)+") 0)";
			if(夾註.classList.contains("sht_gd")){
				夾註.classList.replace("sht_gd","long_gd");
				夾註.insertAdjacentHTML('afterbegin','<span class="gdTailflt"></span>')
				夾註.insertAdjacentHTML('afterbegin','<span class="gdHeadflt"></span>');
				夾註.innerHTML = '<span>'+ 夾註.innerHTML +'</span>';
				if(夾註.nextSibling){
					let 己=document.createElement("span");己.className="sprAftgd";夾註.insertAdjacentElement('afterend',己)//.parentNode.insertBefore(己,夾註.nextSibling);
				}
			}
			let 字距 = 夾註.style.letterSpacing;夾註.style = "";
			夾註.style.width="calc(var(--跨註行寬)*"+String(Math.ceil(哉)*2)+")";夾註.style.letterSpacing = 字距;
			夾註.style.shapeOutside=夾註.style.clipPath=於;
      夾註.getElementsByClassName("gdHeadflt")[0].style.height = 庚;
      夾註.getElementsByClassName("gdTailflt")[0].style.clipPath = 夾註.getElementsByClassName("gdTailflt")[0].style.shapeOutside= 也;
      if(夾註.nextElementSibling && 夾註.nextElementSibling.className=="sprAftgd"){
        夾註.nextElementSibling.style.height=String(乎)+"em";
      }
		}
	}
}
function 單行夾注(夾注樣式類) {
	var cts = document.querySelectorAll('.leftLine, .rightLine, .leftLCmt, .rightLCmt'), cs;
	for(let ct of cts){
		cs = ct.getElementsByTagName("cite");
		for(let c of cs){
			if(c.classList.contains("mh6grug")){
				c.innerHTML = '<span><span>'+ c.innerHTML +'</span></span>';
			} else {
				c.innerHTML = '<span>'+ c.innerHTML +'</span>';
			}
		}
	}
}
function 切䈎(半䈎行數=10, 卷號="inMain", {本書名=undefined,卷名=undefined,版權="哂蟹齋",魚尾式=12}={}){
	contentBlk.style.display = "";
	var 冊䈎 = document.getElementById("冊䈎");
	if(冊䈎===null){
		冊䈎 = document.createElement("div");冊䈎.id = "冊䈎";
		contentBlk.parentNode.insertBefore(冊䈎, contentBlk.nextSibling);
	} else {
		冊䈎.innerHTML = '';
	}
	if(半䈎行數<0){
		冊䈎.remove();
		setTimeout(() => {增刪絲欄();}, 500);
		return;
	}
	var 諸空段 = document.getElementsByClassName("空行");
	for(let 空段 of 諸空段){
		空段.innerHTML = 空段.innerHTML.replace(/<br>/g,'　<br>');
	}
	var 滾者 = window.location.pathname.endsWith('/type/set/') ? document.getElementById("結果") : document.documentElement;
	const 卷長 = 滾者.scrollWidth, 卷進度 = 滾者.scrollLeft/(卷長-滾者.clientWidth);
	滾者.scrollTo(卷長, 0);
	if(!本書名){本書名 = document.getElementById("bookName").textContent.replace(/\s+/g,'');}
	if(本書名.length>7){本書名 = [本書名.slice(0, Math.ceil(本書名.length/2)), 本書名.slice(Math.ceil(本書名.length/2))]}
	if(!卷名){卷名 = document.getElementById("curSec") ? document.getElementById("curSec").textContent.replace(/\s+/g,' ') : '';}
	const 總行數 = document.getElementsByClassName("jienghangsen").length-1;
	const 總半䈎數 = Math.ceil(總行數/半䈎行數);var 總半䈎數補耦 = 總半䈎數;
	if(總半䈎數%2==1){
		總半䈎數補耦++;
	}
	const 卷元素 = document.getElementById(卷號);const 卷類名 = 卷元素.className;
	const 文字區 = document.getElementById("miun").getBoundingClientRect();
	const 文字上沿 = 文字區.top + 正文字號*0.1;//const 文字下沿 = 文字上沿 + 文字區.height;
	const 首段 = 卷元素.children[0];
	var 入版片段 = document.createRange(), 半䈎元素, 版內元素, 版內文, 版內文裡, 天頭元素, 諸䈎片段 = new DocumentFragment();
	入版片段.setStartBefore(首段);入版片段.setEndAfter(首段);
	const 版框高 = document.getElementById("gaeg").style.height, 半䈎版寬 = 半䈎行數*行寬*正文字號;
	var prevScrL = 0, x = 入版片段.getBoundingClientRect().right-1;
	var textNode, offset, range;var 切處所屬, 將刪至 = 首段, 所刪之寬 = 0, 右移量;
	for (let 半䈎敘=1; 半䈎敘<=總半䈎數補耦; 半䈎敘++){
		prevScrL = 滾者.scrollLeft;
		滾者.scrollBy(-半䈎版寬,0);
		if(半䈎敘==總半䈎數){
			切處所屬 = 卷元素.lastElementChild;
		} else if(半䈎敘<總半䈎數) {
			x += prevScrL - 半䈎版寬 - 滾者.scrollLeft;
			if (document.caretPositionFromPoint) {
				range = document.caretPositionFromPoint(x, 文字上沿+2);
				textNode = range.offsetNode;
				offset = range.offset;
			} else if (document.caretRangeFromPoint) {
				range = document.caretRangeFromPoint(x, 文字上沿+2);
				textNode = range.startContainer;
				offset = range.startOffset;
			}
			切處所屬 = textNode.parentElement;
			while(切處所屬.parentElement.id!=卷號){
				切處所屬 = 切處所屬.parentElement;
			}
		}
		版內元素 = document.createElement("div");版內元素.className = "冊䈎版內";版內元素.style.height = "calc("+ 版框高 +" + 0.1em)";
		天頭元素 = document.createElement("div");天頭元素.className = "冊䈎天頭";
		版內文 = document.createElement("div");版內文.className = "版內文";
		if(總半䈎數==總半䈎數補耦 || 半䈎敘<總半䈎數補耦){
			入版片段.setEndAfter(切處所屬);
			版內文裡 = document.createElement("div");if(卷類名){版內文裡.className = 卷類名;}
			版內文裡.style.transform = `translateX(calc(${半䈎行數*(半䈎敘-1) - 所刪之寬/正文字號/行寬}*var(--行寬)))`;
			版內文裡.appendChild(入版片段.cloneContents());版內文.appendChild(版內文裡);
			while(將刪至!=切處所屬){
				所刪之寬 += 將刪至.offsetWidth;
				將刪至 = 將刪至.nextElementSibling;
			}
		}
		版內元素.appendChild(版內文);
		let 絲欄片段 = new DocumentFragment(), 新行;
		for(let 行=1; 行<=半䈎行數; 行++){
			新行 = document.createElement("div");新行.className = "jienghangsen";
			絲欄片段.appendChild(新行);
		}
		let 半䈎絲欄 = document.createElement("div");半䈎絲欄.className = "冊䈎絲欄";
		let 半䈎版框 = document.createElement("div");半䈎版框.className = "版框";半䈎版框.style.height = 版框高;
		let 版心 = document.createElement("div");版心.className = "版心";
		if(typeof 本書名 =='string'){
			版心.innerHTML = '<div style="flex-basis:'+String(本書名.length)+'em">'+本書名+'</div>';
		} else {
			版心.innerHTML = '<div class="版心長書名">'+本書名[(半䈎敘+1)%2]+'</div>';
		}
		版心.innerHTML += '<div><b class="魚尾"></b><span>'+卷名+'</span><span>'+數["讀十萬以內"](Math.ceil(半䈎敘/2))+'</span><b class="魚尾"></b></div><div>'+版權+'</div>';
		半䈎絲欄.appendChild(絲欄片段);半䈎敘%2==1 ? 半䈎絲欄.append(版心) : 半䈎絲欄.prepend(版心);
		半䈎版框.append(半䈎絲欄);版內元素.prepend(半䈎版框);
		半䈎元素 = document.createElement("div");
		半䈎元素.append(天頭元素,版內元素);
		諸䈎片段.appendChild(半䈎元素);
		入版片段.setStartBefore(切處所屬);
	}
	冊䈎.appendChild(諸䈎片段);
	document.querySelector(':root').style.setProperty('--版內文寬', `calc(var(--行寬)*${半䈎行數})`);
	contentBlk.style.display = document.getElementById("right_row").style.display = document.getElementById("left_row").style.display = "none";
	滾者.scrollTo((滾者.scrollWidth-滾者.clientWidth)*卷進度, 0);
	for(let 空段 of 諸空段){
		空段.innerHTML = 空段.innerHTML.replace(/　/g,'');
	}
}
function 卷䈎互轉(){
	if(contentBlk.style.display=="none"){
		切䈎(-9);
	} else {
		var 切誰 = "inMain";const 目次卷 = document.getElementById("miug");
		if(目次卷 && 目次卷.style.display!="none"){
			切誰 = "miug";
		}
		if(document.getElementById("outPDFpan")===null){
			切䈎(10,切誰);
		} else {
			document.getElementById("outPDFpan").style.display = "flex";
			document.getElementById("outPDFwd").contentWindow.切誰 = 切誰;
		}
	}
}