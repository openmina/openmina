import { Injectable } from '@angular/core';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';

@Injectable({
  providedIn: 'root',
})
export class RustService {

  private node: MinaNode;

  constructor(private http: HttpClient) {}

  changeRustNode(node: MinaNode): void {
    this.node = node;
  }

  get URL(): string {
    return this.node.url;
  }

  get name(): string {
    return this.node.name;
  }

  get<T>(path: string): Observable<T> {
    return this.http.get<T>(this.URL + path);
  }
}
