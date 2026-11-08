/* 哂蟹齋排版器（https://vert.neocities.org/type/set/）原文件，經作者許可收錄；路徑已改為相對路徑。 */
'use strict';
var 本選單= window.frameElement.parentElement;
var 切誰 = "inMain";
document.getElementById("okbtn").addEventListener('click',()=>{
	本選單.style.display = "";
	const 半䈎行數 = Number(document.getElementById("每面行數").value);
	window.parent.切䈎(半䈎行數,切誰,{本書名:document.getElementById("版心頂欄字").value,版權:document.getElementById("版心底欄字").value});
});
document.getElementById("canclbtn").addEventListener('click',()=>{
	本選單.style.display = "";
});