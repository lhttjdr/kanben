/* 哂蟹齋排版器（https://vert.neocities.org/type/set/）原文件，經作者許可收錄；路徑已改為相對路徑。 */
'use strict';
var 根 = window.parent.document.querySelector(':root');
var 字號鈕 = document.getElementById("fontSz").getElementsByTagName("A");
var 本選單= window.frameElement;//window.parent.document.getElementById("ctxmenu");
var 行距鈕 = document.getElementById("行距").getElementsByTagName("A");
var 色彩鈕 = document.getElementById("色彩").getElementsByTagName("A");
var 切䈎設置窗 = window.parent.document.getElementById("outPDFwd");
const 行寬選值 = {壁虎:[1.6, 2, 2.4], 其餘:[1.6, 2.4, 3.2]};
var 字號選值 = [20, 15, 10];
if (window.parent.系統 == 'Android' || window.parent.系統 == 'iPhone' || window.parent.系統 == '鴻蒙'){
  document.documentElement.style.fontSize = "25px";
  字號選值 = [30, 25, 20];
}
document.getElementById("選項調疊敘").onclick = function(){
  const 主窗絲欄 = window.parent.document.getElementById("gaeg");
  if(主窗絲欄!==null){
    if(Number(主窗絲欄.style.zIndex)==0 || 主窗絲欄.style.zIndex==""){
      主窗絲欄.style.zIndex = 1;
    } else {
      主窗絲欄.style.zIndex = 0;
    }
  }
  本選單.style.display = "";
}

document.getElementById("選項收展夾注").onclick = function(){
  var 雙行夾注集 = window.parent.document.getElementsByClassName("grebdriuo"), 注後條集 = window.parent.document.getElementsByClassName("sprAftgd");
  var 當歬顯隱 = "";var a = 0;
  for(let i=0; i<雙行夾注集.length; i++){
    當歬顯隱 = window.getComputedStyle(雙行夾注集[i]).getPropertyValue("display");
    if(當歬顯隱 =="none"){
      if(雙行夾注集[i].classList.contains("long_gd")){
        雙行夾注集[i].style.display = "block";
        if(注後條集[a]!==undefined){
          注後條集[a].style.display = "inline-block";a++;
        }
      } else if(雙行夾注集[i].classList.contains("sht_gd") || 雙行夾注集[i].classList.contains("ig_gd")) {
        雙行夾注集[i].style.display = "inline-block";
      }
    } else {
      雙行夾注集[i].style.display = "none";
      if(雙行夾注集[i].classList.contains("long_gd") && 注後條集[a]!==undefined){
        注後條集[a].style.display = "none";a++;
      }
    }
  }
  本選單.style.display = "";
  window.parent.增刪絲欄();
}
function 調整圖(行寬,所選="1"){
  var 諸圖 = window.parent.document.querySelectorAll('figure,fig-il');
  if(所選=="0"){
    for(let 圖 of 諸圖){
      圖.style.width = "";
    }
  } else {
    for(let 圖 of 諸圖){
      let 佔行數 = parseFloat(window.getComputedStyle(圖).getPropertyValue("width"))/window.parent.正文字號/行寬;
      圖.style.width = "calc(var(--行寬)*"+ Math.round(佔行數).toString()+")";
    }
  }
}
function 設字號(l){
  var infos = window.parent.document.getElementById("infos");
  if(infos){
    infos.style.display = "block";
  }
  根.style.setProperty('--基礎字號', String(l+window.parent.設置.字號增減)+"px");//.document.documentElement.style.fontSize = String(l+window.parent.設置.字號增減)+"px";
  window.parent.重排();
  調整圖(window.parent.行寬);
  本選單.style.display = "";
  window.parent.設置.字號 = l;
  if(infos){
    infos.style.display = "";
  }
  本選單.style.fontSize = (字號選值[1]/l).toString()+"em";
}
字號鈕[0].onclick = function(){設字號(字號選值[0]);this.classList.add("選中");字號鈕[1].classList.remove("選中");字號鈕[2].classList.remove("選中");};
字號鈕[1].onclick = function(){設字號(字號選值[1]);字號鈕[0].classList.remove("選中");this.classList.add("選中");字號鈕[2].classList.remove("選中");};
字號鈕[2].onclick = function(){設字號(字號選值[2]);字號鈕[0].classList.remove("選中");字號鈕[1].classList.remove("選中");this.classList.add("選中");};
document.getElementById("重排").addEventListener('click',()=>{
  window.parent.重排();
  本選單.style.display = "";
});
document.getElementById("卷內尋").addEventListener('click', (事)=>{
  本選單.style.display = "";
  window.parent.滑出搜索欄(事);
});
for(let i=0; i<3; i++){
  行距鈕[i].addEventListener('click',function(){
    let 行寬 = window.parent.瀏覽器["名"]=='火狐' ? 行寬選值["壁虎"][Number(this.dataset.lw)] : 行寬選值["其餘"][Number(this.dataset.lw)];
    調整圖(行寬,this.dataset.lw);
    根.style.setProperty('--行寬', String(行寬)+"em");
    window.parent.行寬 = 行寬;window.parent.跨註行寬 = 行寬/1.6;
    for(let k=0; k<3; k++){
      行距鈕[k].classList.remove("選中");
    }
    this.classList.add("選中");
    本選單.style.display = "";
    window.parent.對齊眉批();setTimeout(() => {
      window.parent.增刪絲欄();
    }, 300);
  });
}
function 易配色(){
  const 根之樣式 = window.parent.getComputedStyle(根);
  document.body.style.backgroundColor = 根之樣式.getPropertyValue('--選單背景色');
  document.body.style.color = 根之樣式.getPropertyValue('--主字色');
  if(切䈎設置窗!==null){
    切䈎設置窗.contentDocument.body.style.backgroundColor = 根之樣式.getPropertyValue('--選單背景色');
    切䈎設置窗.contentDocument.body.style.color = 根之樣式.getPropertyValue('--主字色');
  }
}
setTimeout(易配色, 3000);
var 配色表 = window.parent.document.getElementById("配色表");
if(配色表 === null){
  配色表 = window.parent.document.createElement('link');配色表.id = "配色表";配色表.rel = "stylesheet";
  window.parent.document.head.appendChild(配色表);
}
for(let i=0; i<色彩鈕.length; i++){
  色彩鈕[i].addEventListener('click',function(){
    if(this.id.indexOf('原色')<0){
      配色表.href = "../../temas/"+ this.id.slice(0,-1) +".css";
    } else {
      配色表.href = ""; 易配色();
    }
    //根.style.setProperty('--紙色',this.style.backgroundColor);
    for(let k=0; k<色彩鈕.length; k++){
      色彩鈕[k].classList.remove("選中");
    }
    this.classList.add("選中");
    本選單.style.display = "";
  });
}
配色表.addEventListener('load',易配色);
色彩鈕[1].style.backgroundColor = window.parent.getComputedStyle(window.parent.document.getElementById("content_blk")).getPropertyValue("background-color");
document.querySelector(':root').style.setProperty('--原字色', window.parent.getComputedStyle(根).getPropertyValue('--主字色'));
document.getElementById("裝釘").addEventListener('click',()=>{
  本選單.style.display = "";
  window.parent.卷䈎互轉();
});
document.getElementById("定版").addEventListener('click',()=>{
  本選單.style.display = "";
  window.parent.print();
});
window.oncontextmenu = (e) => {  e.preventDefault();}