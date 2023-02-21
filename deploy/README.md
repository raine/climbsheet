## Deployment

### Initial setup

```sh
kustomize edit set image "TEMP=$REGISTRY/climbsheet:latest"
kubectl apply -f yaml
```

### To deploy

```sh
./deploy
```
