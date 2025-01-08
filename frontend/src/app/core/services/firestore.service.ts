import { Injectable } from '@angular/core';
import {
  Firestore,
  CollectionReference,
  collection,
  addDoc,
  doc,
  setDoc,
  updateDoc,
  deleteDoc,
  DocumentData,
} from '@angular/fire/firestore';

@Injectable({
  providedIn: 'root',
})
export class FirestoreService {
  private heartbeatCollection: CollectionReference<DocumentData>;

  constructor(private firestore: Firestore) {
    this.heartbeatCollection = collection(this.firestore, 'heartbeat');
  }

  addHeartbeat(data: any): Promise<void> {
    const docRef = doc(this.heartbeatCollection); // Auto-generated ID
    return setDoc(docRef, data);
  }

  updateHeartbeat(id: string, data: any): Promise<void> {
    const docRef = doc(this.heartbeatCollection, id);
    return updateDoc(docRef, data);
  }

  deleteHeartbeat(id: string): Promise<void> {
    const docRef = doc(this.heartbeatCollection, id);
    return deleteDoc(docRef);
  }

  getHeartbeatCollection() {
    return this.heartbeatCollection;
  }
}
