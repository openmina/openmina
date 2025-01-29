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
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';

@Injectable({
  providedIn: 'root',
})
export class FirestoreService {
  private heartbeatCollection: CollectionReference<DocumentData>;
  private cloudFunctionUrl = 'https://us-central1-webnode-gtm-test.cloudfunctions.net/handleValidationAndStore';

  constructor(private firestore: Firestore,
              private http: HttpClient) {
    this.heartbeatCollection = collection(this.firestore, 'heartbeat');
  }

  addHeartbeat(data: any): Observable<any> {
    console.log('Posting to cloud function:', data);
    return this.http.post(this.cloudFunctionUrl, { data });
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
