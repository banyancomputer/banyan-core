import localforage from 'localforage'

// Note: These are the main things we need right now
export async function putBlob(id: string, blob: Blob, store: LocalForage = localforage): Promise<Blob> {
  return store.setItem(id, blob)
}

export async function getBlob(id: string, store: LocalForage = localforage): Promise<Blob | null> {
  return store.getItem(id)
}

/* istanbul ignore next */
export function createStore(name: string): LocalForage {
  return localforage.createInstance({ name })
}

/* istanbul ignore next */
export async function rm(id: string, store: LocalForage = localforage): Promise<void> {
  return store.removeItem(id)
}

export async function dropStore(store: LocalForage): Promise<void> {
  return store.dropInstance()
}

/* istanbul ignore next */
export async function clear(store?: LocalForage): Promise<void> {
  if(store){
    return dropStore(store)
  } else {
    return localforage.clear()
  }
}

export default {
  createStore,
  getBlob,
  putBlob,
  rm,
  dropStore,
	clear
}
