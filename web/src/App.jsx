import { useState, useRef, useEffect } from 'react'
import './App.css'
import { load_wasm } from './wasm'

function App() {
  const canvasRef = useRef(null);
  let [wasmObj, setWasmObj] = useState(null);
  const [count, setCount] = useState(0)

  useEffect(() => {
    if (count === 0) { return }
    if (wasmObj) {
      wasmObj.cancel_animation();
      wasmObj = null;
    }
    let ignore = false;
    const getWasm = async () => {
      const data = await load_wasm(`eg${count}.wasm`);
      if (!ignore) {
        setWasmObj(data);
      }
    }
    if (wasmObj === null && !ignore) {
      console.log('going to fetch wasm!');
      getWasm();
    }
    return () => {
      ignore = true;
    };
  }, [count])

  useEffect(() => {
    console.log(`calling use effect with: count=${count}`)
    console.log(wasmObj);
    if (count === 0) { return }
    if (!canvasRef || !canvasRef.current || !wasmObj) { return }

    console.log('got em!');
    // console.log(wasmObj);
    // console.log(canvasRef);
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
  }, [count, canvasRef, wasmObj])

  return (
    <>
      <h1>Vite + React</h1>
      <div className="card">
        <button onClick={() => setCount((count) => count + 1)}>
          {count === 0 ? 'None loaded yet' : `eg${count}.wasm`}
        </button>
        <p>
          Click button to load new wasm
        </p>
      </div>
      <canvas ref={canvasRef} id="glcanvas" tabIndex='1' />
    </>
  )
}

export default App
