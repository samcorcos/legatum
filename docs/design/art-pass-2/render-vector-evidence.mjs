// Review-only renderer; run on the cloud box, not the laptop.
import {createRequire} from 'node:module';
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {resolve} from 'node:path';
const root=process.cwd(),out=resolve('docs/design/art-pass-2');
const {chromium}=createRequire(resolve('app/package.json'))('playwright');
const browser=await chromium.launch();const page=await browser.newPage();
await page.setContent('<canvas id="c" width="256" height="256"></canvas>');
const items=JSON.parse(readFileSync(out+'/changed.json','utf8'));
mkdirSync(out+'/vector-after',{recursive:true});
for(const item of items.filter(x=>['icon','crest'].includes(x.kind))){
 const pairs=item.kind==='crest'?[[out+'/before/crest-aztecs.svg',out+'/before/crest-aztecs.png']]:[[resolve(item.before),out+'/before/'+item.slug+'.png'],[resolve(item.target),out+'/vector-after/'+item.slug+'.png']];
 for(const [src,dst] of pairs){
  const svg=readFileSync(src,'utf8').replaceAll('currentColor',item.kind==='crest'?'#c79b49':'#2b2419');
  const data=await page.evaluate(async svg=>{const c=document.getElementById('c'),ctx=c.getContext('2d');ctx.clearRect(0,0,256,256);let i=new Image();i.src='data:image/svg+xml;charset=utf-8,'+encodeURIComponent(svg);await i.decode();ctx.drawImage(i,0,0,256,256);return c.toDataURL('image/png').split(',')[1]},svg);
  writeFileSync(dst,Buffer.from(data,'base64'));
 }
}
await browser.close();
