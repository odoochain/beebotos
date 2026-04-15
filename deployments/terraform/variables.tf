variable "namespace" {
  description = "Kubernetes namespace for BeeBotOS"
  type        = string
  default     = "beebotos"
}

variable "image_repository" {
  description = "Docker image repository"
  type        = string
  default     = "beebotos"
}

variable "image_tag" {
  description = "Docker image tag"
  type        = string
  default     = "v2.0.0"
}

variable "log_level" {
  description = "Application log level"
  type        = string
  default     = "info"
}

variable "gateway_replicas" {
  description = "Number of gateway replicas"
  type        = number
  default     = 2
}

variable "agent_replicas" {
  description = "Number of agent runtime replicas"
  type        = number
  default     = 3
}

variable "database_url" {
  description = "PostgreSQL connection URL"
  type        = string
  sensitive   = true
}

variable "redis_url" {
  description = "Redis connection URL"
  type        = string
  sensitive   = true
}

variable "rpc_url" {
  description = "Blockchain RPC URL"
  type        = string
  default     = "https://rpc.monad.xyz"
}

variable "private_key" {
  description = "Private key for blockchain transactions"
  type        = string
  sensitive   = true
}

variable "jwt_secret" {
  description = "JWT secret for authentication"
  type        = string
  sensitive   = true
}

variable "postgres_password" {
  description = "PostgreSQL password"
  type        = string
  sensitive   = true
}

variable "enable_monitoring" {
  description = "Enable monitoring stack"
  type        = bool
  default     = true
}

variable "enable_ingress" {
  description = "Enable ingress controller"
  type        = bool
  default     = true
}
