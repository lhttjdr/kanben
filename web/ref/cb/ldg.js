/* 哂蟹齋排版器（https://vert.neocities.org/type/set/）原文件，經作者許可收錄；路徑已改為相對路徑。 */
'use strict';
class 哂蟹卷 {
	constructor({正文號="inMain", 天頭號="tentou", 手機頂列數=1, 標點色='標點色', 鏈題級=2, 源}={}){
		this.正文號 = 正文號;
		this.天頭號 = 天頭號;
		this.手機頂列數 = 手機頂列數;
		this.標點色 = 標點色;
		this.源 = 源;
		鏈題元素 = 'h'+ 鏈題級.toString();
	}
	排版(){
		var 天頭凷 = document.getElementById(this.天頭號), 正文凷 = document.getElementById(this.正文號);
		var 排文字 = (procT)=>{
			procT = 寫眉批(procT,this.天頭號);
			procT = toHTML(procT);
			天頭凷.innerHTML = toHTML(天頭凷.innerHTML);
			正文凷.innerHTML = punctuateVH('grebdriuo',true,procT);點行間(this.正文號,this.標點色);點行間(this.天頭號);單行夾注("dhgd");度行();寫旁句();
			雙行夾注();
			punctuateVH('grebdriuo',false,this.正文號);
			if(document.getElementById("infos")!==null){document.getElementById("infos").style.display = "none";}
			增刪絲欄();對齊眉批();
			window.addEventListener('resize',()=>{對齊眉批();增刪絲欄();});
		};
		adaptM(this.手機頂列數);
		if(this.源){
			if(typeof this.源 ==='string'){
				var xhReq = new XMLHttpRequest(), rtxt ='', procT ='';
				xhReq.addEventListener('readystatechange',function(){
					if (this.readyState == XMLHttpRequest.DONE && this.status == 200){
						rtxt = this.responseText;
						排文字(rtxt);
					}
				});
				xhReq.open("GET", this.源, true);
				xhReq.send();
			} else if(typeof this.源 ==='object'){
				if(this.源.tagName =='TEXTAREA'){
					排文字(this.源.value);
				} else {
					const 去雜 = this.源.innerHTML.replace(/<\/div>\s*<div>/g,'\n').replace(/<.+?>/g,'').replace(/^[ \t]+/mg,'');
					排文字(去雜);
				}
			}
		} else {
			var rtxt = 正文凷.innerHTML;
			排文字(rtxt);
		}
	}
}