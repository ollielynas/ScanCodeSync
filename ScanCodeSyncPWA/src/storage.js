// storage.js

export const saveFile = async (filename, content) => {
  const root = await navigator.storage.getDirectory();
  const fileHandle = await root.getFileHandle(filename, { create: true });
  const writable = await fileHandle.createWritable();
  await writable.write(content);
  await writable.close();
};

export const loadFile = async (filename) => {
  const root = await navigator.storage.getDirectory();
  const fileHandle = await root.getFileHandle(filename);
  return fileHandle.getFile();
};

export const loadFileAsText = async (filename) => {
  const file = await loadFile(filename);
  return file.text();
};

export const loadFileAsBlob = async (filename) => {
  return loadFile(filename);
};

export const deleteFile = async (filename) => {
  const root = await navigator.storage.getDirectory();
  await root.removeEntry(filename);
};

export const listFiles = async () => {
  const root = await navigator.storage.getDirectory();
  const entries = [];
  for await (const [name] of root.entries()) {
    entries.push(name);
  }
  return entries;
};
