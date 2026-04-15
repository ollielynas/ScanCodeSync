below is a list of database entries that can be created by a client and should be used to sort files. 

```
{
  <!--deprecated-->
  <!--isMaster: bool-->
  
  isDirector: bool
  isOperator: bool
  
  productionName: String
  
  <!--deprecated-->
  <!--enableOperatorName: bool
  operatorName: String-->
  <!--to be replaced with the ability to name cameras and micrphones with a qr code or audio clip-->
  
  enableSceneName: bool
  sceneName: String
  
  enableTakeNumber: bool
  takeNumber: int
  
  <!--this entry should be entered when the device is showen a qr code. any device showen the qr code will know that it should be renamed.-->
  renameDevice: String
}
```

In the future I would like to add gps support

[[/ScanCodeSyncDesktop/src/data/data_entry.rs]]
[[ScanCodeSyncPWA/src/App.jsx]]
