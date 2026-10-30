/** Packaged Rust/WASM view protocol and the narrow WebExtension API surface. */
type Action = { op: string; [key: string]: unknown };
type DomDescription = string | { tag: string; attrs: [string,string][]; children: DomDescription[]; action?: Action };
interface Presentation { view?: DomDescription; message?: string; retry?: Action; load?: Action }
interface RuntimeData extends Presentation { eligible?: boolean }
interface RuntimeResponse extends Presentation { ok: boolean; data?: RuntimeData | null; error?: string }
interface IncomingMessage { op?: string; event?: string; context?: Presentation; page_url?: string; [key:string]: unknown }
interface BrowserSender { id?: string; url?: string; frameId?: number; tab?: { id?: number } }
interface BrowserTab { id?: number; url?: string }
interface BrowserAPI {
  runtime: {
    id: string;
    getURL(path:string):string;
    sendMessage<T=RuntimeResponse>(message:unknown):Promise<T>;
    onMessage: { addListener(listener:(message:IncomingMessage,sender:BrowserSender,respond:(response:unknown)=>void)=>boolean|void):void };
  };
  storage: { session: {
    get(key:string|null):Promise<Record<string,unknown>>;
    set(value:Record<string,unknown>):Promise<void>;
    remove(keys:string|string[]):Promise<void>;
  }};
  tabs: {
    query(query:{ active:boolean; currentWindow:boolean }):Promise<BrowserTab[]>;
    create(properties:{url:string}):Promise<BrowserTab>;
    update(id:number,properties:{url:string}):Promise<BrowserTab>;
    sendMessage<T=unknown>(id:number,message:unknown):Promise<T>;
  };
}
declare var chrome: BrowserAPI;
declare var browser: BrowserAPI | undefined;
declare var limitlessUI: { send(message:unknown):Promise<RuntimeResponse>; asset_url(path:string):string };
declare var limitlessBridge: {
  record_changed():void;
  random_bytes(size:number):Uint8Array;
  session_get(key:string):Promise<unknown>;
  session_set(key:string,value:unknown):Promise<void>;
  session_remove(keys:string[]):Promise<void>;
  session_all():Promise<Record<string,unknown>>;
  new_tab():Promise<number>;
  navigate_tab(id:number,url:string):Promise<void>;
  show_in_tab(id:number,context:Presentation):Promise<unknown>;
  active_context():Promise<{url:string;notices:unknown[];tab_id:number}|null>;
  schedule_abort(controller:AbortController,ms:number):number;
  cancel_timer(id:number):void;
};
declare module '*limitless_runtime.js' {
  export default function init():Promise<unknown>;
  export function dispatch(request:unknown,sender:unknown):Promise<unknown>;
}
declare module '*limitless_ui.js' { export default function init():Promise<unknown>; }
