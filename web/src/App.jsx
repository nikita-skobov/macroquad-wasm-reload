import { useState, useRef, useEffect, useCallback } from 'react'
import './App.css'
import { load_wasm } from './wasm'

function getWsUri() {
  var loc = window.location, new_uri;
  if (loc.protocol === "https:") {
      new_uri = "wss:";
  } else {
      new_uri = "ws:";
  }
  new_uri += "//" + loc.host;
  new_uri += loc.pathname + "ws";
  return new_uri;
}

function AppWs({ cb }) {
  const ws = useRef(null);
  const uri = getWsUri();

  useEffect(() => {
      ws.current = new WebSocket(uri);
      ws.current.onopen = () => console.log("ws opened");
      ws.current.onclose = () => console.log("ws closed");

      const wsCurrent = ws.current;

      return () => {
          wsCurrent.close();
      };
  }, []);

  useEffect(() => {
    if (!ws.current) return;

    console.log('new websocket? setting up interval again');
    setInterval(() => {
      ws.current.send('test');
    }, 300)
  }, [ws]);

  useEffect(() => {
    console.log('changing onmessage to use new cb');
    ws.current.onmessage = (e) => {
      if (e.data === "true") {
        cb();
      }
    };
  }, [ws, cb]);
}

function App() {
  const canvasRef = useRef(null);
  let [wasmObj, setWasmObj] = useState(null);
  const [isInitialLoad, setInitialLoad] = useState(true);

  useEffect(() => {
    if (isInitialLoad) {
      setInitialLoad(false);
    } else {
      return
    }
    console.log('doing initial load')
    let ignore = false;
    const getWasm = async () => {
      console.log('fetching wasm...');
      const data = await load_wasm(`current.wasm`);
      setWasmObj(data);
    }
    if (wasmObj === null && !ignore) {
      console.log('going to fetch wasm!');
      getWasm();
    }
    return () => {
      ignore = true;
    };
  }, [wasmObj, isInitialLoad, setInitialLoad, setWasmObj])

  useEffect(() => {
    if (!canvasRef || !canvasRef.current || !wasmObj) { return }
    console.log('got new wasm!');
    const canvasCurr = canvasRef.current;
    const glCurr = canvasCurr.getContext('webgl2');
    wasmObj.set_canvas(canvasCurr);
    wasmObj.set_gl(glCurr);
    setTimeout(() => {
      try {
        wasmObj.get_wasm_exports().main();
      } catch (e) {
        console.log('meh sum err');
        console.warn(e);
      }
    }, 1)
  }, [wasmObj, canvasRef])

  const cb = useCallback(() => {
    console.log("IN CB. getting new wasm");
    if (wasmObj) {
      console.log('cancelling previous wasm');
      wasmObj.cancel_animation();
      wasmObj = null;
    }
    const getWasm = async () => {
      const data = await load_wasm(`current.wasm`);
      setWasmObj(data);
    }
    getWasm();
  }, [wasmObj, setWasmObj]);

  return (
    <>
      <AppWs cb={cb}/>
      <canvas ref={canvasRef} id="glcanvas" tabIndex='1' />
    </>
  )
}

export default App
