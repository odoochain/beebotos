terraform {
  required_version = ">= 1.5.0"
  
  required_providers {
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.23"
    }
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.11"
    }
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
  
  backend "s3" {
    bucket = "beebotos-terraform-state"
    key    = "infrastructure/terraform.tfstate"
    region = "us-west-2"
  }
}

provider "kubernetes" {
  config_path = "~/.kube/config"
}

provider "helm" {
  kubernetes {
    config_path = "~/.kube/config"
  }
}

# Namespace
resource "kubernetes_namespace" "beebotos" {
  metadata {
    name = var.namespace
    labels = {
      name = var.namespace
    }
  }
}

# ConfigMap
resource "kubernetes_config_map" "beebotos_config" {
  metadata {
    name      = "beebotos-config"
    namespace = kubernetes_namespace.beebotos.metadata[0].name
  }
  
  data = {
    "gateway.yaml" = templatefile("${path.module}/templates/gateway.yaml", {
      log_level = var.log_level
    })
    "brain.yaml" = templatefile("${path.module}/templates/brain.yaml", {
      qdrant_url = "http://qdrant:6333"
    })
    "chain.yaml" = templatefile("${path.module}/templates/chain.yaml", {
      rpc_url = var.rpc_url
    })
    "dao.yaml" = templatefile("${path.module}/templates/dao.yaml", {})
  }
}

# Secrets
resource "kubernetes_secret" "beebotos_secrets" {
  metadata {
    name      = "beebotos-secrets"
    namespace = kubernetes_namespace.beebotos.metadata[0].name
  }
  
  data = {
    "database-url" = var.database_url
    "redis-url"    = var.redis_url
    "rpc-url"      = var.rpc_url
    "private-key"  = var.private_key
    "jwt-secret"   = var.jwt_secret
  }
}

# PostgreSQL Helm Release
resource "helm_release" "postgresql" {
  name       = "postgresql"
  repository = "https://charts.bitnami.com/bitnami"
  chart      = "postgresql"
  namespace  = kubernetes_namespace.beebotos.metadata[0].name
  version    = "13.2.0"
  
  set {
    name  = "auth.username"
    value = "beebotos"
  }
  
  set {
    name  = "auth.password"
    value = var.postgres_password
  }
  
  set {
    name  = "auth.database"
    value = "beebotos"
  }
  
  set {
    name  = "primary.persistence.size"
    value = "10Gi"
  }
}

# Redis Helm Release
resource "helm_release" "redis" {
  name       = "redis"
  repository = "https://charts.bitnami.com/bitnami"
  chart      = "redis"
  namespace  = kubernetes_namespace.beebotos.metadata[0].name
  version    = "18.5.0"
  
  set {
    name  = "auth.enabled"
    value = "false"
  }
  
  set {
    name  = "master.persistence.size"
    value = "5Gi"
  }
}

# Qdrant Helm Release
resource "helm_release" "qdrant" {
  name       = "qdrant"
  repository = "https://qdrant.github.io/qdrant-helm"
  chart      = "qdrant"
  namespace  = kubernetes_namespace.beebotos.metadata[0].name
  version    = "0.7.0"
  
  set {
    name  = "persistence.size"
    value = "10Gi"
  }
}

# Gateway Deployment
resource "kubernetes_deployment" "gateway" {
  metadata {
    name      = "gateway"
    namespace = kubernetes_namespace.beebotos.metadata[0].name
    labels = {
      app = "gateway"
    }
  }
  
  spec {
    replicas = var.gateway_replicas
    
    selector {
      match_labels = {
        app = "gateway"
      }
    }
    
    template {
      metadata {
        labels = {
          app = "gateway"
        }
      }
      
      spec {
        container {
          name  = "gateway"
          image = "${var.image_repository}/gateway:${var.image_tag}"
          
          port {
            container_port = 8080
            name           = "http"
          }
          
          port {
            container_port = 9090
            name           = "metrics"
          }
          
          env {
            name  = "RUST_LOG"
            value = var.log_level
          }
          
          env {
            name = "DATABASE_URL"
            value_from {
              secret_key_ref {
                name = kubernetes_secret.beebotos_secrets.metadata[0].name
                key  = "database-url"
              }
            }
          }
          
          resources {
            requests = {
              memory = "256Mi"
              cpu    = "250m"
            }
            limits = {
              memory = "512Mi"
              cpu    = "500m"
            }
          }
          
          liveness_probe {
            http_get {
              path = "/health"
              port = 8080
            }
            initial_delay_seconds = 10
            period_seconds        = 30
          }
          
          readiness_probe {
            http_get {
              path = "/ready"
              port = 8080
            }
            initial_delay_seconds = 5
            period_seconds        = 10
          }
        }
      }
    }
  }
}

# Gateway Service
resource "kubernetes_service" "gateway" {
  metadata {
    name      = "gateway"
    namespace = kubernetes_namespace.beebotos.metadata[0].name
  }
  
  spec {
    selector = {
      app = "gateway"
    }
    
    port {
      port        = 80
      target_port = 8080
      name        = "http"
    }
    
    port {
      port        = 9090
      target_port = 9090
      name        = "metrics"
    }
    
    type = "LoadBalancer"
  }
}
