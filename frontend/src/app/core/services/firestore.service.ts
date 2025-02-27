import { Injectable, Optional } from '@angular/core';
import {
  collection,
  CollectionReference,
  deleteDoc,
  doc,
  DocumentData,
  Firestore,
  updateDoc
} from '@angular/fire/firestore';
import { HttpClient } from '@angular/common/http';
import { catchError, Observable, of } from 'rxjs';

@Injectable({
  providedIn: 'root',
})
export class FirestoreService {
  private heartbeatCollection: CollectionReference<DocumentData>;
  private cloudFunctionUrl = 'https://us-central1-webnode-gtm-test.cloudfunctions.net/handleValidationAndStore';

  constructor(@Optional() private firestore: Firestore,
              private http: HttpClient) {
    if (this.firestore) {
      this.heartbeatCollection = collection(this.firestore, 'heartbeat');
    }
  }

  addHeartbeat(data: any): Observable<any> {
    console.log('Posting to cloud function:', data);
    return this.http.post(this.cloudFunctionUrl, { data }).pipe(
      catchError(error => {
        console.error('Error while posting to cloud function:', error);
        return of(null);
      }),
    );
  }

  updateHeartbeat(id: string, data: any): Promise<void> {
    const docRef = doc(this.heartbeatCollection, id);
    return updateDoc(docRef, data);
  }

  deleteHeartbeat(id: string): Promise<void> {
    const docRef = doc(this.heartbeatCollection, id);
    return deleteDoc(docRef);
  }
}
