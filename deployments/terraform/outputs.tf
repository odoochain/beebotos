output "namespace" {
  description = "Kubernetes namespace"
  value       = kubernetes_namespace.beebotos.metadata[0].name
}

output "gateway_service_ip" {
  description = "Gateway service IP address"
  value       = kubernetes_service.gateway.status[0].load_balancer[0].ingress[0].ip
}

output "postgresql_service" {
  description = "PostgreSQL service endpoint"
  value       = "${helm_release.postgresql.name}-postgresql.${kubernetes_namespace.beebotos.metadata[0].name}.svc.cluster.local"
}

output "redis_service" {
  description = "Redis service endpoint"
  value       = "${helm_release.redis.name}-redis-master.${kubernetes_namespace.beebotos.metadata[0].name}.svc.cluster.local:6379"
}

output "qdrant_service" {
  description = "Qdrant service endpoint"
  value       = "${helm_release.qdrant.name}.${kubernetes_namespace.beebotos.metadata[0].name}.svc.cluster.local:6333"
}
