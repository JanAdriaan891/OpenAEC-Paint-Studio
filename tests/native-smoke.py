"""Exercise the built native API on the host OS; optionally open the desktop window."""
import json, os, socket, subprocess, sys, tempfile, time, tomllib, urllib.request
from pathlib import Path
binaries=Path(sys.argv[1]).resolve()
suffix='.exe' if sys.platform=='win32' else ''
with tempfile.TemporaryDirectory() as directory:
 with socket.socket() as s:s.bind(('127.0.0.1',0));port=s.getsockname()[1]
 env=dict(os.environ,PAINT_DATA_DIR=directory,PAINT_API_PORT=str(port),PAINT_API_TOKEN='native-smoke-private-test-token')
 process=subprocess.Popen([str(binaries/('paint-api'+suffix))],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
 base=f'http://127.0.0.1:{port}'
 def request(path,body=None):
  payload=None if body is None else json.dumps(body).encode()
  r=urllib.request.Request(base+path,data=payload,headers={'Authorization':'Bearer native-smoke-private-test-token','Content-Type':'application/json'})
  with urllib.request.urlopen(r,timeout=3) as response:return response.read()
 try:
  for i in range(100):
   try:health=json.loads(request('/health'));break
   except Exception:
    if process.poll() is not None:raise RuntimeError(process.stderr.read().decode())
    time.sleep(.1)
  else:raise RuntimeError('API did not start')
  expected_version=tomllib.loads((Path(__file__).resolve().parent.parent/'Cargo.toml').read_text())['workspace']['package']['version']
  assert health['version']==expected_version,health
  document={'format':'openaec-paint','version':1,'name':'Native smoke','width':64,'height':48,'background':'#ffffff','retainVectors':True,'operations':[]}
  created=json.loads(request('/v1/documents',document))
  request(f"/v1/documents/{created['id']}/operations",{'expectedRevision':1,'operations':[{'type':'rectangle','from':{'x':5,'y':5},'to':{'x':40,'y':35},'color':'#d97706','size':2,'filled':True,'opacity':1}]})
  png=request(f"/v1/documents/{created['id']}/export.png")
  assert png.startswith(b'\x89PNG\r\n\x1a\n'),png[:50]
 finally:
  process.terminate();process.communicate(timeout=10)
if '--gui' in sys.argv:
 # A fresh data directory keeps CI separate from any developer's local draft.
 with tempfile.TemporaryDirectory() as directory:
  env=dict(os.environ,XDG_DATA_HOME=directory,XDG_CONFIG_HOME=directory)
  process=subprocess.Popen([str(binaries/('openaec-paint-studio'+suffix))],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
  try:
   time.sleep(4)
   assert process.poll() is None,process.stderr.read().decode()
  finally:
   if process.poll() is None:process.terminate()
   process.communicate(timeout=10)
print('Native API render smoke passed'+(' and desktop window opened' if '--gui' in sys.argv else ''))
